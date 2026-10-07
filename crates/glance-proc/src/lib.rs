//! Low-overhead `/proc` scanner and safe process signalling.
//!
//! This crate has no GUI dependencies. It reads Linux `/proc` directly
//! (never shells out to `ps`/`top`) in two tiers:
//!
//! * **Tier 1** (every tick, every process): `/proc/<pid>/stat` only.
//! * **Tier 2** (once per process identity): `cmdline`, `exe`, `status`,
//!   `cgroup` and selected `environ` hints.
//!
//! A process identity is [`ProcKey`] — the pair `(pid, start_time)` — so PID
//! reuse never confuses cached data or signal targets.

pub mod parse;
mod read;
pub mod scanner;
pub mod signal;
pub mod types;

pub use scanner::{ScanReport, Scanner};
pub use signal::{PidHandle, Sig, SignalError};
pub use types::{LaunchHints, ProcEntry, ProcKey, ProcSample, ProcStatic, SystemTotals};
