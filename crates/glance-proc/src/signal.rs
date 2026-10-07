//! Race-free process signalling via pidfd.
//!
//! A row in the UI may be seconds old. Sending `kill(pid)` blindly could hit
//! an unrelated process that reused the PID. Instead we:
//!
//! 1. `pidfd_open(pid)` — pins whatever process currently owns the PID;
//! 2. re-read `/proc/<pid>/stat` and check the start time matches the
//!    [`ProcKey`] — if it does, the pinned process is the one we meant;
//! 3. `pidfd_send_signal` on the pinned descriptor.

use std::fmt;
use std::os::fd::OwnedFd;
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::io::Errno;
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

use crate::parse::parse_stat;
use crate::types::ProcKey;

/// Signals Glance can send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sig {
    /// `SIGTERM`: ask the process to exit; it may clean up and save state.
    Term,
    /// `SIGKILL`: stop immediately; unsaved data is lost.
    Kill,
}

impl From<Sig> for Signal {
    fn from(s: Sig) -> Self {
        match s {
            Sig::Term => Signal::TERM,
            Sig::Kill => Signal::KILL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalError {
    /// The process already exited (or the PID now belongs to another process).
    Gone,
    /// Not permitted (another user's process).
    PermissionDenied,
    Os(Errno),
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gone => f.write_str("process already exited"),
            Self::PermissionDenied => f.write_str("permission denied"),
            Self::Os(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SignalError {}

fn map_errno(e: Errno) -> SignalError {
    match e {
        Errno::SRCH => SignalError::Gone,
        Errno::PERM | Errno::ACCESS => SignalError::PermissionDenied,
        other => SignalError::Os(other),
    }
}

/// A verified handle to one specific process.
#[derive(Debug)]
pub struct PidHandle {
    key: ProcKey,
    fd: OwnedFd,
}

impl PidHandle {
    /// Open a handle, verifying the process identity.
    pub fn open(key: ProcKey) -> Result<Self, SignalError> {
        let pid = Pid::from_raw(key.pid).ok_or(SignalError::Gone)?;
        let fd = pidfd_open(pid, PidfdFlags::empty()).map_err(map_errno)?;
        // Verify *after* pinning, so the check cannot be invalidated by reuse.
        let stat = std::fs::read(format!("/proc/{}/stat", key.pid)).map_err(|_| SignalError::Gone)?;
        let st = parse_stat(&stat).ok_or(SignalError::Gone)?;
        if st.start_time != key.start_time {
            return Err(SignalError::Gone);
        }
        Ok(Self { key, fd })
    }

    pub fn key(&self) -> ProcKey {
        self.key
    }

    pub fn signal(&self, sig: Sig) -> Result<(), SignalError> {
        pidfd_send_signal(&self.fd, sig.into()).map_err(map_errno)
    }

    /// Non-blocking exit check.
    pub fn has_exited(&self) -> bool {
        self.wait_exit(Duration::ZERO)
    }

    /// Block up to `timeout` waiting for the process to exit. A pidfd becomes
    /// readable when the process terminates.
    pub fn wait_exit(&self, timeout: Duration) -> bool {
        let ts = Timespec {
            tv_sec: timeout.as_secs() as _,
            tv_nsec: timeout.subsec_nanos() as _,
        };
        let mut fds = [PollFd::new(&self.fd, PollFlags::IN)];
        loop {
            match poll(&mut fds, Some(&ts)) {
                Ok(n) => return n > 0,
                Err(Errno::INTR) => continue,
                Err(_) => return true,
            }
        }
    }
}

/// Convenience: verify and signal in one call.
pub fn send_signal(key: ProcKey, sig: Sig) -> Result<(), SignalError> {
    PidHandle::open(key)?.signal(sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn key_of(pid: u32) -> ProcKey {
        let stat = std::fs::read(format!("/proc/{pid}/stat")).unwrap();
        let st = parse_stat(&stat).unwrap();
        ProcKey { pid: pid as i32, start_time: st.start_time }
    }

    #[test]
    fn terminates_child_and_observes_exit() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let h = PidHandle::open(key_of(child.id())).unwrap();
        assert!(!h.has_exited());
        h.signal(Sig::Term).unwrap();
        // The child is our own, so it stays a zombie until reaped, and a
        // zombie counts as exited for pidfd polling.
        assert!(h.wait_exit(Duration::from_secs(5)));
        let status = child.wait().unwrap();
        assert!(!status.success());
    }

    #[test]
    fn rejects_stale_identity() {
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        let mut key = key_of(child.id());
        key.start_time += 1; // pretend the PID was reused
        assert_eq!(PidHandle::open(key).unwrap_err(), SignalError::Gone);
        child.kill().unwrap();
        child.wait().unwrap();
    }

    #[test]
    fn permission_denied_for_init() {
        if rustix::process::getuid().is_root() {
            return;
        }
        let r = send_signal(key_of(1), Sig::Term);
        assert_eq!(r, Err(SignalError::PermissionDenied));
    }
}
