//! Safety tiers for termination.

use glance_proc::ProcEntry;

/// Ordered from least to most protected; a group takes its members' maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Protection {
    /// Own process: End needs no dialog, Kill asks once.
    #[default]
    Normal,
    /// Another user's process: signals would fail with `EPERM`.
    Elevated,
    /// Desktop-session component: killing it can log the user out.
    Critical,
    /// Never signalled from the UI (PID 1, kernel threads, Glance itself,
    /// the user's systemd manager).
    Blocked,
}

/// Session-critical programs, matched on `comm` (15-byte truncated) and the
/// executable basename. Always combined with UID checks by callers.
const CRITICAL: &[&str] = &[
    "gnome-shell", "gnome-session-b", "gnome-session-binary", "gnome-session-s",
    "gnome-session-service", "gnome-session-c", "gnome-session-ctl", "mutter", "Xorg", "Xwayland",
    "plasmashell", "kwin_wayland", "kwin_x11", "ksmserver", "gdm", "gdm3", "gdm-session-wor",
    "gdm-wayland-ses", "gdm-x-session", "sddm", "lightdm", "dbus-daemon", "dbus-broker",
    "dbus-broker-lau", "dbus-broker-launch", "pipewire", "pipewire-pulse", "wireplumber",
    "systemd-logind", "systemd-journal", "systemd-journald", "systemd-udevd", "NetworkManager",
    "polkitd", "gnome-keyring-d", "gnome-keyring-daemon", "at-spi-bus-laun", "Hyprland", "sway",
    "xfce4-session", "xfwm4", "cinnamon", "cinnamon-sessio", "mate-session",
];

pub fn classify(entry: &ProcEntry, own_uid: u32, own_pid: i32) -> Protection {
    let info = &entry.info;
    let pid = entry.key.pid;
    if pid <= 2 || pid == own_pid || info.is_kernel_thread {
        return Protection::Blocked;
    }
    // The per-user service manager (`systemd --user`).
    if &*info.comm == "systemd" && info.uid == Some(own_uid) {
        return Protection::Blocked;
    }
    if CRITICAL.contains(&&*info.comm)
        || CRITICAL.contains(&info.exe_name())
        || info.cgroup.contains("/session.slice/")
    {
        return Protection::Critical;
    }
    if info.uid != Some(own_uid) {
        return Protection::Elevated;
    }
    // Own processes directly in a login session scope (TTY/SSH shells):
    // ending them may close the session.
    if info.cgroup.rsplit('/').next().is_some_and(|l| l.starts_with("session-") && l.ends_with(".scope")) {
        return Protection::Critical;
    }
    Protection::Normal
}
