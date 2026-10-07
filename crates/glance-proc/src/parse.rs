//! Allocation-free parsers for `/proc` text formats.
//!
//! All parsers take untrusted bytes and return `None` instead of panicking on
//! malformed input.

use crate::types::LaunchHints;

/// `PF_KTHREAD` from `include/linux/sched.h`.
pub const PF_KTHREAD: u64 = 0x0020_0000;

/// Fields of `/proc/<pid>/stat` used by Glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatFields<'a> {
    pub pid: i32,
    pub comm: &'a [u8],
    pub state: u8,
    pub ppid: i32,
    pub flags: u64,
    pub utime: u64,
    pub stime: u64,
    pub threads: u32,
    pub start_time: u64,
    pub rss_pages: u64,
}

/// Parse `/proc/<pid>/stat`.
///
/// `comm` may contain spaces and parentheses, so it is delimited by the first
/// `(` and the **last** `)`.
pub fn parse_stat(buf: &[u8]) -> Option<StatFields<'_>> {
    let open = buf.iter().position(|&b| b == b'(')?;
    let close = buf.iter().rposition(|&b| b == b')')?;
    if close < open {
        return None;
    }
    let pid = parse_u64(trim(&buf[..open]))? as i32;
    let comm = &buf[open + 1..close];
    let rest = buf.get(close + 2..)?;

    // Index 0 is field 3 (state) in proc(5) numbering.
    let mut fields: [&[u8]; 22] = [&[]; 22];
    let mut it = rest.split(|&b| b == b' ');
    for slot in fields.iter_mut() {
        *slot = it.next()?;
    }
    Some(StatFields {
        pid,
        comm,
        state: *fields[0].first()?,
        ppid: parse_u64(fields[1])? as i32,
        flags: parse_u64(fields[6])?,
        utime: parse_u64(fields[11])?,
        stime: parse_u64(fields[12])?,
        threads: parse_u64(fields[17])? as u32,
        start_time: parse_u64(fields[19])?,
        rss_pages: parse_u64(fields[21])?,
    })
}

/// Aggregate CPU counters from the first line of `/proc/stat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuTimes {
    /// user + nice + system + idle + iowait + irq + softirq + steal.
    /// (guest time is already included in user/nice.)
    pub total: u64,
    /// idle + iowait.
    pub idle: u64,
    /// Number of `cpuN` lines.
    pub ncpu: u32,
}

pub fn parse_proc_stat(buf: &[u8]) -> Option<CpuTimes> {
    let mut lines = buf.split(|&b| b == b'\n');
    let first = lines.next()?;
    let rest = first.strip_prefix(b"cpu ")?;
    let mut vals = rest.split(|&b| b == b' ').filter(|s| !s.is_empty()).map(parse_u64);
    let mut v = [0u64; 8];
    for slot in v.iter_mut() {
        *slot = vals.next().flatten().unwrap_or(0);
    }
    let ncpu = lines
        .take_while(|l| l.starts_with(b"cpu"))
        .count() as u32;
    Some(CpuTimes {
        total: v.iter().sum(),
        idle: v[3] + v[4],
        ncpu: ncpu.max(1),
    })
}

/// Look up a `Key:   value kB` line (as in `meminfo`, `status`,
/// `smaps_rollup`) and return the first number.
pub fn find_kv_number(buf: &[u8], key: &[u8]) -> Option<u64> {
    for line in buf.split(|&b| b == b'\n') {
        if let Some(rest) = line.strip_prefix(key)
            && let Some(rest) = rest.strip_prefix(b":")
        {
            let num = trim(rest).split(|&b| b == b' ' || b == b'\t').next()?;
            return parse_u64(num);
        }
    }
    None
}

/// Real UID from `/proc/<pid>/status` (`Uid:\treal\teffective\t...`).
pub fn parse_status_uid(buf: &[u8]) -> Option<u32> {
    find_kv_number(buf, b"Uid").map(|v| v as u32)
}

/// Extract the cgroup path from `/proc/<pid>/cgroup`.
///
/// Prefers the unified (v2) `0::` line, falling back to the v1
/// `name=systemd` hierarchy.
pub fn parse_cgroup(buf: &[u8]) -> Option<&str> {
    let mut v1 = None;
    for line in buf.split(|&b| b == b'\n') {
        if let Some(p) = line.strip_prefix(b"0::") {
            return std::str::from_utf8(p).ok();
        }
        if let Some(idx) = find_sub(line, b":name=systemd:") {
            v1 = std::str::from_utf8(&line[idx + b":name=systemd:".len()..]).ok();
        }
    }
    v1
}

/// Extract desktop-launch hints from a NUL-separated environment block.
pub fn parse_environ_hints(buf: &[u8]) -> LaunchHints {
    let mut h = LaunchHints::default();
    let val = |v: &[u8]| -> Option<Box<str>> {
        std::str::from_utf8(v).ok().filter(|s| !s.is_empty()).map(Into::into)
    };
    for var in buf.split(|&b| b == 0) {
        let Some(eq) = var.iter().position(|&b| b == b'=') else {
            continue;
        };
        let (k, v) = (&var[..eq], &var[eq + 1..]);
        match k {
            b"GIO_LAUNCHED_DESKTOP_FILE" => h.gio_desktop_file = val(v),
            b"GIO_LAUNCHED_DESKTOP_FILE_PID" => h.gio_desktop_pid = parse_u64(v).map(|p| p as i32),
            b"BAMF_DESKTOP_FILE_HINT" => h.bamf_desktop_file = val(v),
            b"FLATPAK_ID" => h.flatpak_id = val(v),
            b"SNAP_NAME" => h.snap_name = val(v),
            _ => {}
        }
    }
    h
}

pub fn parse_u64(s: &[u8]) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut v: u64 = 0;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        v = v.checked_mul(10)?.checked_add(u64::from(b - b'0'))?;
    }
    Some(v)
}

fn trim(s: &[u8]) -> &[u8] {
    s.trim_ascii()
}

fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAT: &[u8] = b"4821 (Web Content) S 4700 4700 4700 0 -1 4194560 155000 0 12 0 \
        1200 340 0 0 20 0 31 0 987654 3000000000 51200 18446744073709551615 1 1 0 0 0 0 0 4096 \
        17663 0 0 0 17 2 0 0 0 0 0\n";

    #[test]
    fn stat_basic() {
        let s = parse_stat(STAT).unwrap();
        assert_eq!(s.pid, 4821);
        assert_eq!(s.comm, b"Web Content");
        assert_eq!(s.state, b'S');
        assert_eq!(s.ppid, 4700);
        assert_eq!(s.utime, 1200);
        assert_eq!(s.stime, 340);
        assert_eq!(s.threads, 31);
        assert_eq!(s.start_time, 987654);
        assert_eq!(s.rss_pages, 51200);
        assert_eq!(s.flags & PF_KTHREAD, 0);
    }

    #[test]
    fn stat_hostile_comm() {
        let buf = b"77 (a) b) (c) R 1 77 77 0 -1 2097216 0 0 0 0 5 6 0 0 20 0 1 0 42 0 0 0\n";
        let s = parse_stat(buf).unwrap();
        assert_eq!(s.comm, b"a) b) (c");
        assert_eq!(s.state, b'R');
        assert_eq!(s.start_time, 42);
        assert_ne!(s.flags & PF_KTHREAD, 0);
    }

    #[test]
    fn stat_truncated_is_none() {
        assert!(parse_stat(b"1 (init) S 0 1").is_none());
        assert!(parse_stat(b"garbage").is_none());
    }

    #[test]
    fn proc_stat() {
        let buf = b"cpu  100 5 50 1000 20 1 2 3 7 0\ncpu0 50 2 25 500 10 0 1 1 0 0\ncpu1 50 3 25 500 10 1 1 2 0 0\nintr 1\n";
        let c = parse_proc_stat(buf).unwrap();
        assert_eq!(c.total, 100 + 5 + 50 + 1000 + 20 + 1 + 2 + 3);
        assert_eq!(c.idle, 1020);
        assert_eq!(c.ncpu, 2);
    }

    #[test]
    fn kv_and_uid() {
        let status = b"Name:\tbash\nUmask:\t0022\nUid:\t1000\t1000\t1000\t1000\nVmRSS:\t   5120 kB\n";
        assert_eq!(parse_status_uid(status), Some(1000));
        assert_eq!(find_kv_number(status, b"VmRSS"), Some(5120));
        assert_eq!(find_kv_number(b"Pss:    123 kB\n", b"Pss"), Some(123));
        assert_eq!(find_kv_number(b"Pss_Anon: 9 kB\n", b"Pss"), None);
    }

    #[test]
    fn cgroup_v2_and_v1() {
        let v2 = b"0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-firefox-123.scope\n";
        assert_eq!(
            parse_cgroup(v2),
            Some("/user.slice/user-1000.slice/user@1000.service/app.slice/app-gnome-firefox-123.scope")
        );
        let v1 = b"12:cpu:/foo\n1:name=systemd:/system.slice/cups.service\n";
        assert_eq!(parse_cgroup(v1), Some("/system.slice/cups.service"));
    }

    #[test]
    fn environ() {
        let env = b"HOME=/home/x\0GIO_LAUNCHED_DESKTOP_FILE=/usr/share/applications/code.desktop\0GIO_LAUNCHED_DESKTOP_FILE_PID=99\0SECRET=hunter2\0SNAP_NAME=firefox\0";
        let h = parse_environ_hints(env);
        assert_eq!(h.gio_desktop_file.as_deref(), Some("/usr/share/applications/code.desktop"));
        assert_eq!(h.gio_desktop_pid, Some(99));
        assert_eq!(h.snap_name.as_deref(), Some("firefox"));
        assert!(h.flatpak_id.is_none());
    }
}
