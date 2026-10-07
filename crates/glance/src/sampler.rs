#![allow(dead_code)]
//! Background sampling thread for Glance.
//!
//! Scans `/proc`, groups processes, and yields snapshots to the GTK thread
//! via an asynchronous channel.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use glance_group::{AppGroup, DesktopIndex, Grouper};
use glance_proc::{ProcEntry, Scanner, SystemTotals};

/// Complete state snapshot sent to the UI thread on each tick.
#[derive(Clone)]
pub struct Snapshot {
    pub groups: Vec<AppGroup>,
    pub procs: std::sync::Arc<HashMap<i32, ProcEntry>>,
    pub totals: SystemTotals,
    pub generation: u64,
    pub scan_time: Duration,
    pub group_time: Duration,
}

/// Commands sent from the GTK main loop to the sampler thread.
pub enum SamplerCommand {
    /// Update refresh interval (e.g. 1000ms when focused, 2000ms when unfocused).
    SetInterval(Duration),
    /// Pause scanning (window minimized or hidden).
    Pause,
    /// Resume scanning (window restored).
    Resume,
    /// Trigger immediate scan (after process termination).
    ImmediateScan,
    /// Request PSS reading for specific visible PIDs.
    RequestPss(Vec<i32>),
}

pub struct SamplerHandle {
    pub cmd_sender: std::sync::mpsc::Sender<SamplerCommand>,
    pub snapshot_receiver: async_channel::Receiver<Snapshot>,
    pub running: Arc<AtomicBool>,
}

pub fn start_sampler() -> SamplerHandle {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<SamplerCommand>();
    let (snap_tx, snap_rx) = async_channel::bounded::<Snapshot>(2);
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    std::thread::Builder::new()
        .name("glance-sampler".into())
        .spawn(move || {
            run_sampler_loop(cmd_rx, snap_tx, running_clone);
        })
        .expect("failed to spawn sampler thread");

    SamplerHandle {
        cmd_sender: cmd_tx,
        snapshot_receiver: snap_rx,
        running,
    }
}

fn run_sampler_loop(
    cmd_rx: std::sync::mpsc::Receiver<SamplerCommand>,
    snap_tx: async_channel::Sender<Snapshot>,
    running: Arc<AtomicBool>,
) {
    let mut scanner = match Scanner::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("glance: failed to initialize /proc scanner: {e}");
            return;
        }
    };
    let index = DesktopIndex::load();
    let mut grouper = Grouper::new(index, scanner.own_uid());

    let mut interval = Duration::from_millis(1000);
    let mut paused = false;

    while running.load(Ordering::Relaxed) {
        if paused {
            // Block until a command arrives. Zero CPU!
            match cmd_rx.recv() {
                Ok(SamplerCommand::Resume) => paused = false,
                Ok(SamplerCommand::SetInterval(i)) => interval = i,
                Ok(SamplerCommand::Pause) => {}
                Ok(SamplerCommand::ImmediateScan) => paused = false,
                Ok(SamplerCommand::RequestPss(pids)) => {
                    scanner.refresh_pss(pids);
                }
                Err(_) => break, // Channel closed
            }
            continue;
        }

        // Perform scan
        let report = match scanner.scan() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("glance: scan error: {e}");
                continue;
            }
        };
        grouper.forget(&report.removed);

        let t_group_start = Instant::now();
        let groups = grouper.group(scanner.entries());
        let group_time = t_group_start.elapsed();

        let snapshot = Snapshot {
            groups,
            procs: Arc::new(scanner.entries().clone()),
            totals: scanner.totals(),
            generation: scanner.generation(),
            scan_time: report.duration,
            group_time,
        };

        // Send to GTK thread; if buffer full, drop older snapshot so UI doesn't lag
        let _ = snap_tx.try_send(snapshot);

        // Sleep until next scan or command arrives
        let wait_start = Instant::now();
        while let Some(remaining) = interval.checked_sub(wait_start.elapsed()) {
            if remaining.is_zero() {
                break;
            }
            match cmd_rx.recv_timeout(remaining) {
                Ok(SamplerCommand::Pause) => {
                    paused = true;
                    break;
                }
                Ok(SamplerCommand::Resume) => {
                    paused = false;
                }
                Ok(SamplerCommand::SetInterval(i)) => {
                    interval = i;
                }
                Ok(SamplerCommand::ImmediateScan) => {
                    break;
                }
                Ok(SamplerCommand::RequestPss(pids)) => {
                    scanner.refresh_pss(pids);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return;
                }
            }
        }
    }
}
