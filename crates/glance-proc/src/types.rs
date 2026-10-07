//! Core process data types.

use std::path::Path;
use std::sync::Arc;

/// Stable identity of a process.
///
/// PIDs wrap around, so a bare PID is never used as an identity. The start
/// time (clock ticks since boot, field 22 of `/proc/<pid>/stat`) disambiguates
/// a reused PID.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProcKey {
    pub pid: i32,
    pub start_time: u64,
}

/// Desktop-launch hints extracted from `/proc/<pid>/environ`.
///
/// Only readable for processes owned by the current user. Every other
/// environment variable is discarded immediately after parsing.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LaunchHints {
    /// `GIO_LAUNCHED_DESKTOP_FILE` (path to the `.desktop` file).
    pub gio_desktop_file: Option<Box<str>>,
    /// `GIO_LAUNCHED_DESKTOP_FILE_PID`: the PID GIO actually launched. The
    /// variable is inherited by descendants, so the hint is only authoritative
    /// for the process whose PID matches.
    pub gio_desktop_pid: Option<i32>,
    /// `BAMF_DESKTOP_FILE_HINT` (set by snap launchers and Unity-era tooling).
    pub bamf_desktop_file: Option<Box<str>>,
    /// `FLATPAK_ID`.
    pub flatpak_id: Option<Box<str>>,
    /// `SNAP_NAME`.
    pub snap_name: Option<Box<str>>,
}

impl LaunchHints {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Data that is fixed for the lifetime of a process (read once, Tier 2).
#[derive(Debug, Clone)]
pub struct ProcStatic {
    /// Kernel `comm` (at most 15 bytes, may be truncated).
    pub comm: Box<str>,
    /// Raw NUL-separated argv. Empty for kernel threads and zombies.
    pub cmdline: Box<[u8]>,
    /// Resolved `/proc/<pid>/exe`. `None` when unreadable (other users,
    /// kernel threads).
    pub exe: Option<Box<Path>>,
    /// Real UID. `None` if the process vanished before `status` was read.
    pub uid: Option<u32>,
    /// cgroup v2 path (or the v1 `name=systemd` hierarchy as a fallback).
    pub cgroup: Box<str>,
    pub hints: LaunchHints,
    pub is_kernel_thread: bool,
}

impl ProcStatic {
    /// Iterate argv entries as raw bytes.
    pub fn argv(&self) -> impl Iterator<Item = &[u8]> {
        self.cmdline.split(|&b| b == 0).filter(|s| !s.is_empty())
    }

    /// argv joined with spaces, lossily decoded and with control characters
    /// replaced, safe to show in a plain-text label.
    pub fn display_cmdline(&self) -> String {
        let mut out = String::with_capacity(self.cmdline.len());
        for (i, arg) in self.argv().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            for ch in String::from_utf8_lossy(arg).chars() {
                out.push(if ch.is_control() { '\u{FFFD}' } else { ch });
            }
        }
        out
    }

    /// Basename of the executable, falling back to `comm`.
    pub fn exe_name(&self) -> &str {
        self.exe
            .as_deref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or(&self.comm)
    }
}

/// Per-tick sample (Tier 1).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProcSample {
    pub ppid: i32,
    /// Single-letter state from `stat` (`R`, `S`, `D`, `Z`, ...).
    pub state: u8,
    /// User + system CPU time in clock ticks.
    pub cpu_ticks: u64,
    pub rss_kib: u64,
    pub threads: u32,
}

/// A live process as tracked by the [`crate::Scanner`].
#[derive(Debug, Clone)]
pub struct ProcEntry {
    pub key: ProcKey,
    pub info: Arc<ProcStatic>,
    pub sample: ProcSample,
    /// Share of total machine CPU over the last interval, in permille
    /// (0..=1000). `None` until two samples exist.
    pub cpu_permille: Option<u16>,
    /// Proportional set size from `smaps_rollup`, refreshed on demand.
    pub pss_kib: Option<u64>,
    pub(crate) last_seen: u64,
}

impl ProcEntry {
    /// Build an entry outside the scanner (fixtures, tests, replay).
    pub fn new(key: ProcKey, info: Arc<ProcStatic>, sample: ProcSample) -> Self {
        Self { key, info, sample, cpu_permille: None, pss_kib: None, last_seen: 0 }
    }
}

/// Machine-wide totals computed during a scan.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SystemTotals {
    /// Busy share of all CPUs over the last interval, permille.
    pub cpu_permille: u16,
    pub mem_total_kib: u64,
    pub mem_available_kib: u64,
    pub ncpu: u32,
    pub process_count: u32,
}
