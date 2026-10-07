//! The `/proc` scanner.

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::fd::OwnedFd;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustix::fs::{Dir, Mode, OFlags, open, openat, readlinkat};
use rustix::io;

use crate::parse::{
    CpuTimes, PF_KTHREAD, find_kv_number, parse_cgroup, parse_environ_hints, parse_proc_stat,
    parse_stat, parse_status_uid, parse_u64,
};
use crate::read::{PidPath, read_file};
use crate::types::{LaunchHints, ProcEntry, ProcKey, ProcSample, ProcStatic, SystemTotals};

const STAT_MAX: usize = 8 * 1024;
const CMDLINE_MAX: usize = 128 * 1024;
const ENVIRON_MAX: usize = 256 * 1024;
const SMALL_MAX: usize = 64 * 1024;

/// Result of one [`Scanner::scan`] call.
#[derive(Debug, Default, Clone)]
pub struct ScanReport {
    pub generation: u64,
    pub added: Vec<ProcKey>,
    pub removed: Vec<ProcKey>,
    pub duration: Duration,
}

/// Incremental `/proc` scanner.
///
/// Keeps a table of live processes keyed by PID. Each [`scan`](Self::scan)
/// performs one `readdir` of `/proc` plus one `stat` read per process; static
/// details are read only when a new process identity appears.
pub struct Scanner {
    proc_dir: OwnedFd,
    dir: Dir,
    buf: Vec<u8>,
    path: PidPath,
    entries: HashMap<i32, ProcEntry>,
    generation: u64,
    prev_cpu: Option<CpuTimes>,
    page_kib: u64,
    own_uid: u32,
    totals: SystemTotals,
    /// Open `/proc/<pid>/stat` descriptors, re-read with `pread` each tick.
    /// Saves an `openat` + `close` (and the path walk) per process.
    stat_fds: HashMap<i32, OwnedFd>,
    cache_fds: bool,
}

/// Upper bound on cached descriptors (beyond it, fall back to open/close).
const MAX_CACHED_FDS: usize = 8192;

impl Scanner {
    /// Open the system `/proc`.
    pub fn new() -> io::Result<Self> {
        Self::with_root("/proc")
    }

    /// Open an alternative procfs root (used by tests with fixture trees).
    pub fn with_root(root: impl AsRef<Path>) -> io::Result<Self> {
        let proc_dir = open(
            root.as_ref(),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let dir = Dir::read_from(&proc_dir)?;
        Ok(Self {
            proc_dir,
            dir,
            buf: vec![0; 4096],
            path: PidPath::new(),
            entries: HashMap::with_capacity(512),
            generation: 0,
            prev_cpu: None,
            page_kib: (rustix::param::page_size() / 1024) as u64,
            own_uid: rustix::process::getuid().as_raw(),
            totals: SystemTotals::default(),
            stat_fds: HashMap::with_capacity(512),
            cache_fds: raise_nofile_limit(),
        })
    }

    pub fn entries(&self) -> &HashMap<i32, ProcEntry> {
        &self.entries
    }

    pub fn get(&self, pid: i32) -> Option<&ProcEntry> {
        self.entries.get(&pid)
    }

    pub fn totals(&self) -> SystemTotals {
        self.totals
    }

    pub fn own_uid(&self) -> u32 {
        self.own_uid
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Sample every process once.
    pub fn scan(&mut self) -> io::Result<ScanReport> {
        let started = Instant::now();
        self.generation += 1;
        let generation = self.generation;

        // Machine-wide CPU counters. CPU shares are computed against the
        // total jiffies of all CPUs, so 1000 permille == the whole machine.
        let cpu = read_file(&self.proc_dir, c"stat", &mut self.buf, SMALL_MAX)
            .ok()
            .and_then(|n| parse_proc_stat(&self.buf[..n]));
        let cpu_delta = match (self.prev_cpu, cpu) {
            (Some(p), Some(c)) if c.total > p.total => {
                Some((c.total - p.total, c.idle.saturating_sub(p.idle)))
            }
            _ => None,
        };
        if cpu.is_some() {
            self.prev_cpu = cpu;
        }
        if let Some(c) = cpu {
            self.totals.ncpu = c.ncpu;
        }
        if let Some((total, idle)) = cpu_delta {
            self.totals.cpu_permille = permille(total.saturating_sub(idle), total);
        }
        if let Ok(n) = read_file(&self.proc_dir, c"meminfo", &mut self.buf, SMALL_MAX) {
            let m = &self.buf[..n];
            self.totals.mem_total_kib = find_kv_number(m, b"MemTotal").unwrap_or(0);
            self.totals.mem_available_kib = find_kv_number(m, b"MemAvailable").unwrap_or(0);
        }

        let mut report = ScanReport { generation, ..Default::default() };
        self.dir.rewind();
        while let Some(ent) = self.dir.read() {
            let ent = ent?;
            let Some(pid) = parse_u64(ent.file_name().to_bytes()).map(|p| p as i32) else {
                continue;
            };
            // The process may exit at any moment; every failure below just
            // means "gone", not an error.
            let Some(n) = self.read_stat(pid) else {
                continue;
            };
            let Some(st) = parse_stat(&self.buf[..n]) else {
                continue;
            };
            let key = ProcKey { pid, start_time: st.start_time };
            let sample = ProcSample {
                ppid: st.ppid,
                state: st.state,
                cpu_ticks: st.utime + st.stime,
                rss_kib: st.rss_pages * self.page_kib,
                threads: st.threads,
            };

            if let Some(e) = self.entries.get_mut(&pid)
                && e.key == key
            {
                e.cpu_permille = cpu_delta.map(|(total, _)| {
                    permille(sample.cpu_ticks.saturating_sub(e.sample.cpu_ticks), total)
                });
                e.sample = sample;
                e.last_seen = generation;
                continue;
            }

            let flags = st.flags;
            let comm = lossy(st.comm);
            let info = self.read_static(pid, comm, flags);
            let entry = ProcEntry {
                key,
                info,
                sample,
                cpu_permille: None,
                pss_kib: None,
                last_seen: generation,
            };
            if let Some(old) = self.entries.insert(pid, entry) {
                // Same PID, different start time: the PID was reused.
                report.removed.push(old.key);
            }
            report.added.push(key);
        }

        let removed = &mut report.removed;
        let stat_fds = &mut self.stat_fds;
        self.entries.retain(|pid, e| {
            let alive = e.last_seen == generation;
            if !alive {
                removed.push(e.key);
                stat_fds.remove(pid);
            }
            alive
        });
        self.totals.process_count = self.entries.len() as u32;
        report.duration = started.elapsed();
        Ok(report)
    }

    /// Refresh proportional set size (PSS) for the given PIDs from
    /// `smaps_rollup`. This is much more expensive than `stat`, so callers
    /// should rate-limit it and restrict it to what is on screen. Returns the
    /// number of processes successfully updated.
    pub fn refresh_pss(&mut self, pids: impl IntoIterator<Item = i32>) -> usize {
        let mut updated = 0;
        for pid in pids {
            if !self.entries.contains_key(&pid) {
                continue;
            }
            let pss = read_file(
                &self.proc_dir,
                self.path.get(pid, "smaps_rollup"),
                &mut self.buf,
                SMALL_MAX,
            )
            .ok()
            .and_then(|n| find_kv_number(&self.buf[..n], b"Pss"));
            if let Some(e) = self.entries.get_mut(&pid) {
                e.pss_kib = pss;
                updated += usize::from(pss.is_some());
            }
        }
        updated
    }

    /// Read `/proc/<pid>/stat` into `self.buf`.
    fn read_stat(&mut self, pid: i32) -> Option<usize> {
        if self.cache_fds {
            if let Some(fd) = self.stat_fds.get(&pid) {
                // A descriptor of an exited task fails with ESRCH (or reads
                // 0 bytes); the PID may have been reused, so reopen.
                match pread_all(fd, &mut self.buf) {
                    Ok(n) if n > 0 => return Some(n),
                    _ => {
                        self.stat_fds.remove(&pid);
                    }
                }
            }
            if self.stat_fds.len() < MAX_CACHED_FDS {
                match openat(
                    &self.proc_dir,
                    self.path.get(pid, "stat"),
                    OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOCTTY,
                    Mode::empty(),
                ) {
                    Ok(fd) => {
                        let n = pread_all(&fd, &mut self.buf).ok().filter(|&n| n > 0)?;
                        self.stat_fds.insert(pid, fd);
                        return Some(n);
                    }
                    Err(io::Errno::MFILE) | Err(io::Errno::NFILE) => {
                        // Out of descriptors: stop caching, release them.
                        self.cache_fds = false;
                        self.stat_fds.clear();
                    }
                    Err(_) => return None,
                }
            }
        }
        read_file(&self.proc_dir, self.path.get(pid, "stat"), &mut self.buf, STAT_MAX).ok()
    }

    fn read_static(&mut self, pid: i32, comm: Box<str>, flags: u64) -> Arc<ProcStatic> {
        if flags & PF_KTHREAD != 0 {
            return Arc::new(ProcStatic {
                comm,
                cmdline: Box::default(),
                exe: None,
                uid: Some(0),
                cgroup: "/".into(),
                hints: LaunchHints::default(),
                is_kernel_thread: true,
            });
        }

        let cmdline: Box<[u8]> =
            match read_file(&self.proc_dir, self.path.get(pid, "cmdline"), &mut self.buf, CMDLINE_MAX) {
                Ok(n) => self.buf[..n].trim_ascii_end_nul().into(),
                Err(_) => Box::default(),
            };

        let exe = readlinkat(&self.proc_dir, self.path.get(pid, "exe"), Vec::new())
            .ok()
            .map(|c| {
                let mut bytes = c.into_bytes();
                // The kernel appends " (deleted)" when the binary was
                // replaced on disk (common right after package upgrades).
                if bytes.ends_with(b" (deleted)") {
                    bytes.truncate(bytes.len() - b" (deleted)".len());
                }
                PathBuf::from(OsString::from_vec(bytes)).into_boxed_path()
            });

        let uid = read_file(&self.proc_dir, self.path.get(pid, "status"), &mut self.buf, SMALL_MAX)
            .ok()
            .and_then(|n| parse_status_uid(&self.buf[..n]));

        let cgroup: Box<str> =
            read_file(&self.proc_dir, self.path.get(pid, "cgroup"), &mut self.buf, SMALL_MAX)
                .ok()
                .and_then(|n| parse_cgroup(&self.buf[..n]).map(Into::into))
                .unwrap_or_default();

        // environ is only readable for our own processes; skip the syscall
        // otherwise. Everything except the launch hints is dropped here.
        let hints = if uid == Some(self.own_uid) {
            read_file(&self.proc_dir, self.path.get(pid, "environ"), &mut self.buf, ENVIRON_MAX)
                .map(|n| parse_environ_hints(&self.buf[..n]))
                .unwrap_or_default()
        } else {
            LaunchHints::default()
        };

        Arc::new(ProcStatic {
            comm,
            cmdline,
            exe,
            uid,
            cgroup,
            hints,
            is_kernel_thread: false,
        })
    }
}

fn pread_all(fd: &OwnedFd, buf: &mut Vec<u8>) -> io::Result<usize> {
    let mut len = 0;
    loop {
        if len == buf.len() {
            if buf.len() >= STAT_MAX {
                return Ok(len);
            }
            buf.resize(buf.len() * 2, 0);
        }
        match io::pread(fd, &mut buf[len..], len as u64) {
            Ok(0) => return Ok(len),
            Ok(n) => len += n,
            Err(io::Errno::INTR) => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Raise the soft `RLIMIT_NOFILE` towards the hard limit so cached stat
/// descriptors fit. Returns whether caching should be enabled.
fn raise_nofile_limit() -> bool {
    use rustix::process::{Resource, getrlimit, setrlimit};
    const WANT: u64 = 16 * 1024;
    let lim = getrlimit(Resource::Nofile);
    let cur = lim.current.unwrap_or(u64::MAX);
    if cur >= WANT {
        return true;
    }
    let max = lim.maximum.unwrap_or(u64::MAX);
    let new = WANT.min(max);
    if new > cur {
        let _ = setrlimit(Resource::Nofile, rustix::process::Rlimit { current: Some(new), maximum: lim.maximum });
    }
    getrlimit(Resource::Nofile).current.unwrap_or(u64::MAX) >= 4096
}

fn permille(part: u64, whole: u64) -> u16 {
    if whole == 0 {
        return 0;
    }
    (part.saturating_mul(1000) / whole).min(1000) as u16
}

fn lossy(bytes: &[u8]) -> Box<str> {
    String::from_utf8_lossy(bytes).into()
}

trait TrimNul {
    fn trim_ascii_end_nul(&self) -> &[u8];
}

impl TrimNul for [u8] {
    fn trim_ascii_end_nul(&self) -> &[u8] {
        let mut s = self;
        while let [rest @ .., 0] = s {
            s = rest;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_live_proc() {
        let mut s = Scanner::new().expect("open /proc");
        let r1 = s.scan().unwrap();
        assert!(!r1.added.is_empty());
        let me = std::process::id() as i32;
        let e = s.get(me).expect("own process present");
        assert_eq!(e.info.uid, Some(s.own_uid()));
        assert!(e.info.exe.is_some());
        assert!(e.cpu_permille.is_none(), "first sample has no delta");

        // Burn a little CPU so the second sample has something to measure.
        let t = Instant::now();
        let mut x = 0u64;
        while t.elapsed() < Duration::from_millis(50) {
            x = x.wrapping_add(std::hint::black_box(1));
        }
        std::hint::black_box(x);

        let r2 = s.scan().unwrap();
        assert_eq!(r2.generation, 2);
        assert!(s.get(me).unwrap().cpu_permille.is_some());
        assert!(s.totals().mem_total_kib > 0);
        assert_eq!(s.refresh_pss([me]), 1);
        assert!(s.get(me).unwrap().pss_kib.unwrap() > 0);
    }

    #[test]
    fn detects_kernel_threads() {
        let mut s = Scanner::new().unwrap();
        s.scan().unwrap();
        // kthreadd is PID 2 in the initial PID namespace.
        if let Some(k) = s.get(2) {
            assert!(k.info.is_kernel_thread);
        }
    }

    #[test]
    fn permille_math() {
        assert_eq!(permille(0, 0), 0);
        assert_eq!(permille(5, 100), 50);
        assert_eq!(permille(200, 100), 1000);
    }
}
