//! Interpret cgroup v2 paths produced by systemd.
//!
//! Desktop environments that follow the freedesktop/systemd "desktop
//! environment integration" convention start every application in its own
//! transient unit:
//!
//! * `app[-<launcher>]-<ApplicationID>-<RANDOM>.scope`
//! * `app[-<launcher>]-<ApplicationID>[@<RANDOM>].service`
//!
//! Dashes inside `<ApplicationID>` are escaped as `\x2d`, so the raw unit name
//! can be split on `-` *before* unescaping.

/// What the leaf unit of a process's cgroup says about it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Freedesktop application unit.
    App { launcher: Option<String>, app_id: String },
    Flatpak { app_id: String },
    /// `snap.<name>.<app>-<uuid>.scope` or `snap.<name>.<app>.service`.
    Snap { name: String, app: String, service: bool },
    /// Per-tab terminal scope, e.g. `vte-spawn-<uuid>.scope`
    /// (GNOME Terminal/Console) or `ptyxis-spawn-<uuid>.scope`.
    Terminal { kind: String, id: String },
    /// D-Bus activated service, `dbus-:1.2-org.foo.Bar@0.service`.
    DbusActivated { bus_name: String },
    /// Any other `.service`.
    Service { unit: String, user: bool },
    /// Login session (`session-3.scope`): TTY, SSH or display-manager logins.
    Session { unit: String },
    /// Container runtime scope (`docker-<id>.scope`, `libpod-<id>.scope`).
    Container { runtime: String, id: String },
    /// PID 1's `init.scope`.
    Init,
    /// Root cgroup (`/`), e.g. kernel threads.
    Root,
    /// Anything else.
    Other { unit: String, user: bool },
}

/// The raw (still escaped) leaf unit name in a cgroup path, if any.
pub fn leaf_unit(path: &str) -> Option<&str> {
    path.rsplit('/')
        .find(|c| c.ends_with(".scope") || c.ends_with(".service"))
}

/// Whether the path lives inside a per-user systemd manager.
pub fn is_user_path(path: &str) -> bool {
    path.contains("/user@")
}

pub fn parse_scope(path: &str) -> Scope {
    if path.is_empty() || path == "/" {
        return Scope::Root;
    }
    let user = is_user_path(path);
    let Some(leaf) = leaf_unit(path) else {
        let last = path.rsplit('/').find(|c| !c.is_empty()).unwrap_or(path);
        return Scope::Other { unit: unescape(last), user };
    };
    if leaf == "init.scope" {
        return Scope::Init;
    }
    let (stem, is_scope) = match leaf.strip_suffix(".scope") {
        Some(s) => (s, true),
        None => (leaf.strip_suffix(".service").unwrap_or(leaf), false),
    };

    if let Some(rest) = stem.strip_prefix("app-") {
        let (launcher, raw_id) = split_app_unit(rest, is_scope);
        let app_id = unescape(&raw_id);
        if launcher.as_deref() == Some("flatpak") {
            return Scope::Flatpak { app_id };
        }
        if let Some(bus) = dbus_bus_name(&app_id) {
            return Scope::DbusActivated { bus_name: bus.to_owned() };
        }
        return Scope::App { launcher, app_id };
    }

    if let Some(rest) = stem.strip_prefix("snap.") {
        let mut it = rest.splitn(2, '.');
        let name = it.next().unwrap_or_default();
        let mut app = it.next().unwrap_or(name);
        if is_scope {
            // `<app>-<uuid>`: strip the random suffix. App names may contain
            // dashes, so prefer cutting exactly one 36-char UUID.
            if let Some(i) = app.len().checked_sub(37)
                && app.as_bytes()[i] == b'-'
            {
                app = &app[..i];
            } else if let Some(i) = app.rfind('-') {
                app = &app[..i];
            }
        }
        return Scope::Snap { name: unescape(name), app: unescape(app), service: !is_scope };
    }

    let unescaped = unescape(stem);
    if let Some(bus) = dbus_bus_name(&unescaped) {
        return Scope::DbusActivated { bus_name: bus.to_owned() };
    }

    if is_scope {
        if let Some(i) = stem.find("-spawn-") {
            return Scope::Terminal { kind: stem[..i].to_owned(), id: stem[i + 7..].to_owned() };
        }
        if stem.starts_with("session-") {
            return Scope::Session { unit: leaf.to_owned() };
        }
        for (prefix, runtime) in [("docker-", "docker"), ("libpod-", "podman"), ("crio-", "cri-o")] {
            if let Some(id) = stem.strip_prefix(prefix) {
                return Scope::Container { runtime: runtime.to_owned(), id: id.to_owned() };
            }
        }
        return Scope::Other { unit: unescape(leaf), user };
    }
    Scope::Service { unit: unescape(leaf), user }
}

/// Split `[launcher-]id[-random]` / `[launcher-]id[@random]`.
fn split_app_unit(rest: &str, is_scope: bool) -> (Option<String>, String) {
    let rest = if is_scope {
        rest
    } else {
        rest.split('@').next().unwrap_or(rest)
    };
    let mut parts: Vec<&str> = rest.split('-').collect();
    if is_scope && parts.len() >= 2 && parts.last().is_some_and(|p| looks_random(p)) {
        parts.pop();
    }
    match parts.as_slice() {
        [] => (None, String::new()),
        [id] => (None, (*id).to_owned()),
        [launcher, id @ ..] => (Some((*launcher).to_owned()), id.join("-")),
    }
}

fn looks_random(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// `dbus-:1.2-org.gnome.Foo` (optionally `@0`) to `org.gnome.Foo`.
fn dbus_bus_name(s: &str) -> Option<&str> {
    let rest = s.strip_prefix("dbus-:")?;
    let after = &rest[rest.find('-')? + 1..];
    Some(after.split('@').next().unwrap_or(after))
}

/// Undo systemd unit-name escaping (`\xNN`).
pub fn unescape(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\'
            && i + 3 < b.len()
            && b[i + 1] == b'x'
            && let (Some(h), Some(l)) = (hex(b[i + 2]), hex(b[i + 3]))
        {
            out.push(h << 4 | l);
            i += 4;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/";

    #[test]
    fn gnome_app_scope() {
        assert_eq!(
            parse_scope(&format!("{U}app-gnome-google\\x2dchrome-4821.scope")),
            Scope::App { launcher: Some("gnome".into()), app_id: "google-chrome".into() }
        );
        assert_eq!(
            parse_scope(&format!("{U}app-gnome-xdg\\x2dterminal\\x2dexec-4002505.scope")),
            Scope::App { launcher: Some("gnome".into()), app_id: "xdg-terminal-exec".into() }
        );
    }

    #[test]
    fn launcherless_scope_and_service() {
        assert_eq!(
            parse_scope(&format!("{U}app-org.chromium.Chromium-42710.scope")),
            Scope::App { launcher: None, app_id: "org.chromium.Chromium".into() }
        );
        assert_eq!(
            parse_scope(&format!("{U}app-org.kde.dolphin@a1b2c3.service")),
            Scope::App { launcher: None, app_id: "org.kde.dolphin".into() }
        );
        assert_eq!(
            parse_scope(&format!("{U}app-kde-org.kde.konsole@x.service")),
            Scope::App { launcher: Some("kde".into()), app_id: "org.kde.konsole".into() }
        );
    }

    #[test]
    fn flatpak_snap_terminal() {
        assert_eq!(
            parse_scope(&format!("{U}app-flatpak-org.mozilla.firefox-12345.scope")),
            Scope::Flatpak { app_id: "org.mozilla.firefox".into() }
        );
        assert_eq!(
            parse_scope(&format!("{U}snap.firefox.firefox-0f3c1a2b-1111-2222-3333-444455556666.scope")),
            Scope::Snap { name: "firefox".into(), app: "firefox".into(), service: false }
        );
        assert_eq!(
            parse_scope(&format!("{U}ptyxis-spawn-53f60796-a421-479e-b0ce-4157cb1143d3.scope")),
            Scope::Terminal { kind: "ptyxis".into(), id: "53f60796-a421-479e-b0ce-4157cb1143d3".into() }
        );
    }

    #[test]
    fn services_sessions_containers() {
        assert_eq!(
            parse_scope("/system.slice/cups.service"),
            Scope::Service { unit: "cups.service".into(), user: false }
        );
        assert_eq!(
            parse_scope("/system.slice/systemd-udevd.service/udev"),
            Scope::Service { unit: "systemd-udevd.service".into(), user: false }
        );
        assert_eq!(
            parse_scope("/user.slice/user-1000.slice/user@1000.service/session.slice/org.gnome.Shell@ubuntu.service"),
            Scope::Service { unit: "org.gnome.Shell@ubuntu.service".into(), user: true }
        );
        assert_eq!(
            parse_scope("/user.slice/user-1000.slice/session-3.scope"),
            Scope::Session { unit: "session-3.scope".into() }
        );
        assert_eq!(
            parse_scope("/system.slice/docker-f17e118c0bc6.scope"),
            Scope::Container { runtime: "docker".into(), id: "f17e118c0bc6".into() }
        );
        assert_eq!(parse_scope("/init.scope"), Scope::Init);
        assert_eq!(parse_scope("/"), Scope::Root);
        assert_eq!(parse_scope(""), Scope::Root);
    }

    #[test]
    fn dbus_activated() {
        assert_eq!(
            parse_scope(&format!("{U}dbus-:1.2-org.gnome.Calculator.SearchProvider@0.service")),
            Scope::DbusActivated { bus_name: "org.gnome.Calculator.SearchProvider".into() }
        );
        assert_eq!(
            parse_scope(&format!("{U}app-dbus\\x2d:1.3\\x2dorg.gnome.Nautilus@0.service")),
            Scope::DbusActivated { bus_name: "org.gnome.Nautilus".into() }
        );
    }

    #[test]
    fn unescape_works() {
        assert_eq!(unescape("a\\x2db\\x20c"), "a-b c");
        assert_eq!(unescape("trailing\\x2"), "trailing\\x2");
    }
}
