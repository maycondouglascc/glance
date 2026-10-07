//! Small syscall helpers: stack-built relative paths and buffered reads
//! relative to an open `/proc` directory descriptor.

use std::ffi::CStr;
use std::os::fd::AsFd;

use rustix::fs::{Mode, OFlags, openat};
use rustix::io::{self, Errno};

/// Builds `"<pid>/<suffix>\0"` on the stack, so hot-path opens never allocate.
pub(crate) struct PidPath {
    buf: [u8; 64],
}

impl PidPath {
    pub(crate) fn new() -> Self {
        Self { buf: [0; 64] }
    }

    /// Returns a C string valid until the next call.
    pub(crate) fn get(&mut self, pid: i32, suffix: &str) -> &CStr {
        let mut digits = [0u8; 12];
        let mut n = pid.unsigned_abs();
        let mut i = digits.len();
        loop {
            i -= 1;
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            if n == 0 {
                break;
            }
        }
        let num = &digits[i..];
        let total = num.len() + 1 + suffix.len();
        assert!(total < self.buf.len(), "suffix too long");
        self.buf[..num.len()].copy_from_slice(num);
        self.buf[num.len()] = b'/';
        self.buf[num.len() + 1..total].copy_from_slice(suffix.as_bytes());
        self.buf[total] = 0;
        // Interior bytes are digits, '/', and a fixed ASCII
        // suffix without NULs, terminated by the single NUL above.
        CStr::from_bytes_with_nul(&self.buf[..=total]).expect("valid path")
    }
}

/// Read a whole file relative to `dir` into `buf`, growing it up to `max`
/// bytes. Returns the number of bytes read. The buffer is reused across
/// calls so steady-state reads do not allocate.
pub(crate) fn read_file<Fd: AsFd>(
    dir: Fd,
    path: &CStr,
    buf: &mut Vec<u8>,
    max: usize,
) -> io::Result<usize> {
    let fd = openat(dir, path, OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOCTTY, Mode::empty())?;
    if buf.len() < 4096 {
        buf.resize(4096, 0);
    }
    let mut len = 0;
    loop {
        if len == buf.len() {
            if buf.len() >= max {
                break;
            }
            let new_len = (buf.len() * 2).min(max);
            buf.resize(new_len, 0);
        }
        match io::read(&fd, &mut buf[len..]) {
            Ok(0) => break,
            Ok(n) => len += n,
            Err(Errno::INTR) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pid_path() {
        let mut p = PidPath::new();
        assert_eq!(p.get(1234, "stat").to_bytes(), b"1234/stat");
        assert_eq!(p.get(0, "exe").to_bytes(), b"0/exe");
        assert_eq!(p.get(i32::MAX, "smaps_rollup").to_bytes(), b"2147483647/smaps_rollup");
    }
}
