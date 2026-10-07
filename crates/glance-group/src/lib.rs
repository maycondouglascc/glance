//! Process-to-application grouping.
//!
//! Turns the flat process table produced by [`glance_proc::Scanner`] into
//! application groups using a ranked resolver chain (see [`resolver`]):
//!
//! 1. systemd app scopes/services in the cgroup path,
//! 2. Flatpak / Snap / terminal-tab / container scopes,
//! 3. desktop-launch environment hints,
//! 4. executable matches against the `.desktop` index,
//! 5. inheritance from the parent process,
//! 6. systemd unit, then executable, as fallbacks.

pub mod cgroup;
pub mod desktop;
pub mod format;
pub mod labels;
pub mod protection;
pub mod resolver;

pub use cgroup::{Scope, parse_scope};
pub use desktop::{DesktopEntry, DesktopIndex};
pub use protection::Protection;
pub use resolver::{AppGroup, AppId, Grouper, Section};
