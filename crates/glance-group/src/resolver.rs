//! Ranked resolver chain: assign every process to an application group.
//!
//! Each process first gets an *intrinsic* resolution from its own static data
//! (cached per [`ProcKey`]), with a strength:
//!
//! * **Strong**: systemd app scope, Flatpak, Snap, terminal tab, container,
//!   or a GIO launch hint addressed to this exact PID.
//! * **Medium**: executable matches a `.desktop` entry.
//! * **Weak**: systemd unit or bare executable fallback.
//!
//! The final assignment then walks the process tree: Strong and Medium win,
//! Weak processes inherit an app-like parent group unless a session boundary
//! (`systemd`, `gnome-shell`, ...) separates them.

use std::collections::HashMap;
use std::sync::Arc;

use glance_proc::{ProcEntry, ProcKey};

use crate::cgroup::{Scope, is_user_path, leaf_unit, parse_scope};
use crate::desktop::DesktopIndex;
use crate::labels::{process_label, script_name};
use crate::protection::{Protection, classify};

/// Identity of an application group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AppId {
    /// A `.desktop` application (ID without the `.desktop` suffix), or a
    /// freedesktop app-scope ID without an installed entry.
    Desktop(Arc<str>),
    Flatpak(Arc<str>),
    Snap(Arc<str>),
    /// One terminal tab scope.
    Terminal(Arc<str>),
    Container(Arc<str>),
    Unit { name: Arc<str>, user: bool },
    Exe { path: Arc<str>, uid: u32 },
    Kernel,
}

impl AppId {
    fn is_app_like(&self) -> bool {
        matches!(self, Self::Desktop(_) | Self::Flatpak(_) | Self::Snap(_) | Self::Terminal(_) | Self::Container(_))
    }
}

/// Top-level sections, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    Apps,
    Background,
    System,
    Kernel,
}

impl Section {
    pub fn title(self) -> &'static str {
        match self {
            Self::Apps => "Apps",
            Self::Background => "Background",
            Self::System => "System",
            Self::Kernel => "Kernel",
        }
    }
}

/// One application with its member processes.
#[derive(Debug, Clone)]
pub struct AppGroup {
    pub id: AppId,
    pub section: Section,
    pub name: String,
    pub icon: Option<Arc<str>>,
    /// Short qualifier shown next to the name ("Terminal", "Snap", ...).
    pub badge: Option<&'static str>,
    /// Members, sorted by PID.
    pub members: Vec<ProcKey>,
    /// Members whose parent is outside the group: graceful-quit targets.
    pub roots: Vec<ProcKey>,
    pub cpu_permille: u32,
    /// PSS where available, RSS otherwise.
    pub mem_kib: u64,
    /// True when every member contributed PSS (accurate, no double-counting
    /// of shared pages).
    pub mem_is_pss: bool,
    pub protection: Protection,
    /// User-manager unit that contains exactly this group, usable with
    /// `org.freedesktop.systemd1.Manager.KillUnit`.
    pub unit: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Strength {
    Weak,
    Medium,
    Strong,
}

#[derive(Debug, Clone)]
struct Intrinsic {
    id: AppId,
    strength: Strength,
    /// Do not inherit groups *from* this process.
    boundary: bool,
    label: Arc<str>,
    protection: Protection,
    /// App-scope ID with no installed `.desktop` entry, if any.
    unknown_scope: Option<Arc<str>>,
}

/// Programs that start other programs on the user's behalf. Children of
/// these are not part of the launcher's group.
const BOUNDARIES: &[&str] = &[
    "systemd", "init", "gnome-shell", "plasmashell", "kwin_wayland", "kwin_x11", "krunner",
    "xfce4-panel", "xfdesktop", "xfce4-session", "lxpanel", "sshd", "sshd-session", "login",
    "gdm-session-wor", "gnome-session-b", "dbus-daemon", "dbus-broker", "dbus-broker-lau",
    "containerd-shim", "containerd-shim-runc-v2", "bwrap", "xdg-desktop-por", "nautilus",
];

const SHELLS: &[&str] = &["bash", "sh", "dash", "zsh", "fish", "ksh", "tcsh", "nu", "login"];

/// Stateful grouper; caches per-process resolution across ticks.
pub struct Grouper {
    index: DesktopIndex,
    own_uid: u32,
    own_pid: i32,
    cache: HashMap<ProcKey, Intrinsic>,
}

impl Grouper {
    pub fn new(index: DesktopIndex, own_uid: u32) -> Self {
        Self { index, own_uid, own_pid: std::process::id() as i32, cache: HashMap::new() }
    }

    pub fn index(&self) -> &DesktopIndex {
        &self.index
    }

    /// Replace the desktop index (after `.desktop` files change) and drop
    /// cached resolutions.
    pub fn set_index(&mut self, index: DesktopIndex) {
        self.index = index;
        self.cache.clear();
    }

    /// Drop cached data for processes that exited.
    pub fn forget(&mut self, removed: &[ProcKey]) {
        for k in removed {
            self.cache.remove(k);
        }
    }

    /// Friendly label for a process seen in the last [`group`](Self::group).
    pub fn label(&self, key: &ProcKey) -> Option<&str> {
        self.cache.get(key).map(|i| &*i.label)
    }

    pub fn protection(&self, key: &ProcKey) -> Protection {
        self.cache.get(key).map_or(Protection::Normal, |i| i.protection)
    }

    /// Group the current process table.
    pub fn group(&mut self, entries: &HashMap<i32, ProcEntry>) -> Vec<AppGroup> {
        for e in entries.values() {
            if !self.cache.contains_key(&e.key) {
                let intr = self.intrinsic(e);
                self.cache.insert(e.key, intr);
            }
        }

        let mut assigned: HashMap<i32, AppId> = HashMap::with_capacity(entries.len());
        for &pid in entries.keys() {
            self.resolve(pid, entries, &mut assigned, 0);
        }

        // An app scope whose ID has no `.desktop` entry (e.g. an Electron app
        // launching helpers under a generic Chromium ID) adopts the app that
        // any member's executable matched, so every member of the scope joins.
        let mut scope_map: HashMap<Arc<str>, AppId> = HashMap::new();
        for e in entries.values() {
            let intr = &self.cache[&e.key];
            if let Some(s) = &intr.unknown_scope
                && !matches!(&intr.id, AppId::Desktop(d) if d == s)
            {
                scope_map.entry(s.clone()).or_insert_with(|| intr.id.clone());
            }
        }
        if !scope_map.is_empty() {
            for id in assigned.values_mut() {
                if let AppId::Desktop(s) = id
                    && let Some(target) = scope_map.get(s)
                {
                    *id = target.clone();
                }
            }
        }

        let mut groups: HashMap<AppId, AppGroup> = HashMap::new();
        let mut pss_complete: HashMap<AppId, bool> = HashMap::new();
        let mut units: HashMap<AppId, Option<Option<&str>>> = HashMap::new();
        for (pid, e) in entries {
            let id = assigned[pid].clone();
            let intr = &self.cache[&e.key];
            let g = groups.entry(id.clone()).or_insert_with(|| AppGroup {
                id: id.clone(),
                section: Section::Apps,
                name: String::new(),
                icon: None,
                badge: None,
                members: Vec::new(),
                roots: Vec::new(),
                cpu_permille: 0,
                mem_kib: 0,
                mem_is_pss: true,
                protection: Protection::Normal,
                unit: None,
            });
            g.members.push(e.key);
            g.cpu_permille += u32::from(e.cpu_permille.unwrap_or(0));
            g.mem_kib += e.pss_kib.unwrap_or(e.sample.rss_kib);
            if e.pss_kib.is_none() && !e.info.is_kernel_thread {
                pss_complete.insert(id.clone(), false);
            }
            g.protection = g.protection.max(intr.protection);
            if entries.get(&e.sample.ppid).is_none_or(|_| assigned.get(&e.sample.ppid) != Some(&id)) {
                g.roots.push(e.key);
            }
            // Track whether all members share one user-manager unit.
            let leaf = is_user_path(&e.info.cgroup).then(|| leaf_unit(&e.info.cgroup)).flatten();
            units
                .entry(id)
                .and_modify(|u| {
                    if *u != Some(leaf) {
                        *u = Some(None);
                    }
                })
                .or_insert(Some(leaf));
        }

        let mut out: Vec<AppGroup> = groups.into_values().collect();
        for g in &mut out {
            g.members.sort();
            g.roots.sort();
            g.mem_is_pss = pss_complete.get(&g.id).copied().unwrap_or(true);
            g.unit = units.get(&g.id).copied().flatten().flatten().map(Into::into);
            self.describe(g, entries);
        }
        out.sort_by(|a, b| a.section.cmp(&b.section).then_with(|| a.name.cmp(&b.name)));
        out
    }

    fn resolve(&self, pid: i32, entries: &HashMap<i32, ProcEntry>, memo: &mut HashMap<i32, AppId>, depth: u32) -> AppId {
        if let Some(id) = memo.get(&pid) {
            return id.clone();
        }
        let e = &entries[&pid];
        let intr = &self.cache[&e.key];
        let id = if intr.strength >= Strength::Medium || depth > 64 {
            intr.id.clone()
        } else {
            let parent = entries.get(&e.sample.ppid).filter(|p| p.key.pid != pid);
            match parent {
                Some(p) if !self.cache[&p.key].boundary => {
                    let pid_parent = p.key.pid;
                    let pf = self.resolve(pid_parent, entries, memo, depth + 1);
                    let inherit = match &intr.id {
                        // cgroup says which service it is; trust it unless
                        // the parent belongs to an app.
                        AppId::Unit { .. } => pf.is_app_like(),
                        AppId::Exe { uid, .. } => pf.is_app_like() || p.info.uid == Some(*uid),
                        _ => false,
                    };
                    if inherit { pf } else { intr.id.clone() }
                }
                _ => intr.id.clone(),
            }
        };
        memo.insert(pid, id.clone());
        id
    }

    fn intrinsic(&self, e: &ProcEntry) -> Intrinsic {
        let info = &e.info;
        let label: Arc<str> = process_label(info).into();
        let protection = classify(e, self.own_uid, self.own_pid);
        let boundary = e.key.pid == 1
            || BOUNDARIES.contains(&&*info.comm)
            || BOUNDARIES.contains(&info.exe_name());
        let mk = |id: AppId, strength: Strength| Intrinsic {
            id,
            strength,
            boundary,
            label: label.clone(),
            protection,
            unknown_scope: None,
        };
        if info.is_kernel_thread {
            return mk(AppId::Kernel, Strength::Strong);
        }
        let uid = info.uid.unwrap_or(u32::MAX);
        let scope = parse_scope(&info.cgroup);

        // 1-2. cgroup scopes.
        match &scope {
            Scope::App { app_id, .. } => {
                if let Some(d) = self.index.by_id(app_id) {
                    return mk(desktop_id(&d.id), Strength::Strong);
                }
                // Scope without installed entry (e.g. a launcher helper):
                // prefer an executable match, else keep the scope's ID.
                let scope_id: Arc<str> = app_id.as_str().into();
                let id = self.match_exe(e).unwrap_or_else(|| AppId::Desktop(scope_id.clone()));
                let mut intr = mk(id, Strength::Strong);
                intr.unknown_scope = Some(scope_id);
                return intr;
            }
            Scope::Flatpak { app_id } => return mk(AppId::Flatpak(app_id.as_str().into()), Strength::Strong),
            Scope::Snap { name, .. } => return mk(AppId::Snap(name.as_str().into()), Strength::Strong),
            Scope::Terminal { id, .. } => return mk(AppId::Terminal(id.as_str().into()), Strength::Strong),
            Scope::Container { id, .. } => return mk(AppId::Container(id.as_str().into()), Strength::Strong),
            Scope::DbusActivated { bus_name } => {
                if let Some(d) = self.index.by_id(bus_name).filter(|d| !d.no_display) {
                    return mk(desktop_id(&d.id), Strength::Strong);
                }
            }
            _ => {}
        }

        // 3. Launch hints from the environment.
        let h = &info.hints;
        if let Some(f) = h.gio_desktop_file.as_deref()
            && h.gio_desktop_pid == Some(e.key.pid)
            && let Some(d) = self.index.by_id(f.rsplit('/').next().unwrap_or(f))
        {
            return mk(desktop_id(&d.id), Strength::Strong);
        }
        if let Some(id) = h.flatpak_id.as_deref() {
            return mk(AppId::Flatpak(id.into()), Strength::Strong);
        }
        if let Some(name) = h.snap_name.as_deref() {
            return mk(AppId::Snap(name.into()), Strength::Strong);
        }

        // 4. Executable matches.
        if let Some(id) = self.match_exe(e) {
            return mk(id, Strength::Medium);
        }

        // 5-6. Fallbacks (inheritance is applied in `resolve`).
        match scope {
            Scope::Service { unit, user } => mk(AppId::Unit { name: unit.into(), user }, Strength::Weak),
            Scope::DbusActivated { bus_name } => mk(
                AppId::Unit { name: bus_name.into(), user: is_user_path(&info.cgroup) },
                Strength::Weak,
            ),
            _ => {
                let path = info
                    .exe
                    .as_deref()
                    .and_then(|p| p.to_str())
                    .map(Into::into)
                    .unwrap_or_else(|| info.comm.clone().into());
                mk(AppId::Exe { path, uid }, Strength::Weak)
            }
        }
    }

    fn match_exe(&self, e: &ProcEntry) -> Option<AppId> {
        let info = &e.info;
        let exe = info.exe.as_deref();
        let d = exe
            .and_then(|p| self.index.by_exe(p))
            .or_else(|| {
                // Interpreted apps: match the script path or name.
                let script_path = info
                    .argv()
                    .skip(1)
                    .map(String::from_utf8_lossy)
                    .find(|a| a.starts_with('/'))?;
                self.index.by_exe(std::path::Path::new(&*script_path))
            })
            .or_else(|| exe.and_then(|p| p.file_name()?.to_str()).and_then(|n| self.index.by_name(n)))
            .or_else(|| script_name(info).and_then(|s| self.index.by_name(&s)))?;
        Some(desktop_id(&d.id))
    }

    /// Fill in display name, icon, badge and section.
    fn describe(&self, g: &mut AppGroup, entries: &HashMap<i32, ProcEntry>) {
        let first = g.roots.first().or(g.members.first()).and_then(|k| entries.get(&k.pid));
        let uid = first.and_then(|e| e.info.uid);
        let own = uid == Some(self.own_uid);
        let bg_or_sys = if own { Section::Background } else { Section::System };
        let root_label = || first.and_then(|e| self.label(&e.key)).unwrap_or("?").to_owned();

        match &g.id {
            AppId::Kernel => {
                g.name = "Kernel Threads".into();
                g.section = Section::Kernel;
            }
            AppId::Desktop(id) => match self.index.by_id(id) {
                Some(d) => {
                    g.name = d.name.to_string();
                    g.icon = d.icon.clone();
                    g.section = if d.no_display { bg_or_sys } else { Section::Apps };
                }
                None => {
                    // App scope with no installed entry: usually autostart
                    // helpers (update-notifier, gsd notifiers), not apps.
                    g.name = id.to_string();
                    g.section = bg_or_sys;
                }
            },
            AppId::Flatpak(id) => {
                g.badge = Some("Flatpak");
                match self.index.by_id(id) {
                    Some(d) => {
                        g.name = d.name.to_string();
                        g.icon = d.icon.clone();
                        g.section = if d.no_display { bg_or_sys } else { Section::Apps };
                    }
                    None => {
                        g.name = id.rsplit('.').next().unwrap_or(id).to_owned();
                        g.section = bg_or_sys;
                    }
                }
            }
            AppId::Snap(name) => {
                g.badge = Some("Snap");
                let d = self
                    .index
                    .by_id(&format!("{name}_{name}"))
                    .or_else(|| self.index.by_id_prefix(&format!("{name}_")));
                // Snap daemons/user services run as `.service` units.
                let is_service = first.is_some_and(|e| e.info.cgroup.ends_with(".service"));
                match d {
                    Some(d) if !d.no_display && !is_service => {
                        g.name = d.name.to_string();
                        g.icon = d.icon.clone();
                        g.section = Section::Apps;
                    }
                    Some(d) => {
                        g.name = d.name.to_string();
                        g.icon = d.icon.clone();
                        g.section = bg_or_sys;
                    }
                    None => {
                        g.name = name.to_string();
                        g.section = bg_or_sys;
                    }
                }
            }
            AppId::Terminal(_) => {
                g.badge = Some("Terminal");
                g.section = Section::Apps;
                g.name = self.terminal_name(g, entries);
                g.icon = Some(self.terminal_icon(first).into());
            }
            AppId::Container(id) => {
                g.badge = Some("Container");
                g.section = Section::System;
                g.name = format!("Container {}", &id[..id.len().min(12)]);
                g.icon = Some("application-x-executable".into());
            }
            AppId::Unit { name, user } => {
                g.badge = Some("Service");
                let stem = name.trim_end_matches(".service");
                g.name = stem.split('@').next().unwrap_or(stem).to_owned();
                g.section = if *user && own { Section::Background } else { Section::System };
            }
            AppId::Exe { .. } => {
                g.name = root_label();
                g.section = bg_or_sys;
            }
        }
    }

    /// Name a terminal tab after its most significant non-shell process:
    /// the earliest-started one whose parent is a shell or outside the tab.
    fn terminal_name(&self, g: &AppGroup, entries: &HashMap<i32, ProcEntry>) -> String {
        let is_shell = |e: &ProcEntry| SHELLS.contains(&&*e.info.comm) || SHELLS.contains(&e.info.exe_name());
        let in_group: std::collections::HashSet<i32> = g.members.iter().map(|k| k.pid).collect();
        let fg = g
            .members
            .iter()
            .filter_map(|k| entries.get(&k.pid))
            .filter(|e| !is_shell(e))
            .filter(|e| {
                !in_group.contains(&e.sample.ppid) || entries.get(&e.sample.ppid).is_some_and(is_shell)
            })
            .min_by_key(|e| e.key.start_time);
        let pick = fg.or_else(|| g.roots.first().and_then(|k| entries.get(&k.pid)));
        pick.and_then(|e| self.label(&e.key)).unwrap_or("Terminal").to_owned()
    }

    fn terminal_icon(&self, first: Option<&ProcEntry>) -> String {
        if let Some(e) = first
            && let Scope::Terminal { kind, .. } = parse_scope(&e.info.cgroup)
        {
            let guess = match kind.as_str() {
                "vte" => self.index.by_name("gnome-terminal").or_else(|| self.index.by_name("kgx")),
                k => self.index.by_name(k),
            };
            if let Some(icon) = guess.and_then(|d| d.icon.clone()) {
                return icon.to_string();
            }
        }
        "utilities-terminal".into()
    }
}

fn desktop_id(id: &str) -> AppId {
    AppId::Desktop(id.strip_suffix(".desktop").unwrap_or(id).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glance_proc::{LaunchHints, ProcSample, ProcStatic};

    const UAPP: &str = "/user.slice/user-1000.slice/user@1000.service/app.slice/";

    struct Builder(HashMap<i32, ProcEntry>);

    impl Builder {
        fn add(&mut self, pid: i32, ppid: i32, comm: &str, exe: &str, argv: &[&str], cgroup: &str, uid: u32) {
            let info = ProcStatic {
                comm: comm.into(),
                cmdline: argv.join("\0").into_bytes().into(),
                exe: (!exe.is_empty()).then(|| std::path::Path::new(exe).into()),
                uid: Some(uid),
                cgroup: cgroup.into(),
                hints: LaunchHints::default(),
                is_kernel_thread: false,
            };
            let mut e = test_entry(pid, info);
            e.sample.ppid = ppid;
            e.sample.rss_kib = 1000;
            e.cpu_permille = Some(10);
            self.0.insert(pid, e);
        }
    }

    fn test_entry(pid: i32, info: ProcStatic) -> ProcEntry {
        ProcEntry::new(ProcKey { pid, start_time: pid as u64 }, Arc::new(info), ProcSample::default())
    }

    fn index() -> DesktopIndex {
        let mut idx = DesktopIndex::default();
        idx.insert_text(
            "google-chrome.desktop",
            "[Desktop Entry]\nType=Application\nName=Google Chrome\nIcon=google-chrome\nExec=/opt/google/chrome/chrome %U\n",
        );
        idx.insert_text(
            "code.desktop",
            "[Desktop Entry]\nType=Application\nName=Visual Studio Code\nIcon=vscode\nExec=/usr/share/code/code %F\n",
        );
        idx.insert_text(
            "firefox_firefox.desktop",
            "[Desktop Entry]\nType=Application\nName=Firefox\nIcon=firefox\nExec=/snap/bin/firefox %u\n",
        );
        idx
    }

    fn find<'a>(gs: &'a [AppGroup], name: &str) -> &'a AppGroup {
        gs.iter().find(|g| g.name == name).unwrap_or_else(|| panic!("no group {name}: {gs:#?}"))
    }

    #[test]
    fn groups_by_scope_tree_and_exe() {
        let mut b = Builder(HashMap::new());
        let chrome_scope = format!("{UAPP}app-gnome-google\\x2dchrome-100.scope");
        b.add(50, 1, "systemd", "/usr/lib/systemd/systemd", &["/usr/lib/systemd/systemd", "--user"], "/user.slice/user-1000.slice/user@1000.service/init.scope", 1000);
        b.add(100, 50, "chrome", "/opt/google/chrome/chrome", &["/opt/google/chrome/chrome"], &chrome_scope, 1000);
        b.add(101, 100, "chrome", "/opt/google/chrome/chrome", &["/opt/google/chrome/chrome", "--type=renderer"], &chrome_scope, 1000);
        b.add(102, 100, "chrome_crashpad", "/opt/google/chrome/chrome_crashpad_handler", &["x"], &chrome_scope, 1000);
        // VS Code started without a scope (e.g. from a script): exe match + inheritance.
        b.add(200, 50, "code", "/usr/share/code/code", &["/usr/share/code/code"], &format!("{UAPP}foo.service"), 1000);
        b.add(201, 200, "git", "/usr/bin/git", &["git", "status"], &format!("{UAPP}foo.service"), 1000);
        // Snap Firefox.
        let snap = format!("{UAPP}snap.firefox.firefox-0f3c1a2b-1111-2222-3333-444455556666.scope");
        b.add(300, 50, "firefox", "/snap/firefox/1/usr/lib/firefox/firefox", &["firefox"], &snap, 1000);
        b.add(301, 300, "Isolated Web Co", "/snap/firefox/1/usr/lib/firefox/firefox", &["firefox", "-contentproc", "tab"], &snap, 1000);
        // Terminal tab running cargo.
        let tab = format!("{UAPP}ptyxis-spawn-53f60796-a421-479e-b0ce-4157cb1143d3.scope");
        b.add(400, 50, "bash", "/usr/bin/bash", &["/usr/bin/bash"], &tab, 1000);
        b.add(401, 400, "cargo", "/home/u/.cargo/bin/cargo", &["cargo", "build"], &tab, 1000);
        b.add(402, 401, "rustc", "/home/u/.rustup/x/rustc", &["rustc"], &tab, 1000);
        // System service.
        b.add(500, 1, "cupsd", "/usr/sbin/cupsd", &["/usr/sbin/cupsd", "-l"], "/system.slice/cups.service", 0);
        b.add(1, 0, "systemd", "/usr/lib/systemd/systemd", &["/sbin/init"], "/init.scope", 0);

        let mut g = Grouper::new(index(), 1000);
        let gs = g.group(&b.0);

        let chrome = find(&gs, "Google Chrome");
        assert_eq!(chrome.members.len(), 3);
        assert_eq!(chrome.section, Section::Apps);
        assert_eq!(chrome.roots.len(), 1);
        assert_eq!(chrome.roots[0].pid, 100);
        assert_eq!(chrome.cpu_permille, 30);
        assert_eq!(chrome.unit.as_deref(), Some("app-gnome-google\\x2dchrome-100.scope"));
        assert_eq!(g.label(&chrome.members[1]), Some("Renderer"));

        let code = find(&gs, "Visual Studio Code");
        assert_eq!(code.members.len(), 2, "git inherits VS Code");

        let ff = find(&gs, "Firefox");
        assert_eq!(ff.members.len(), 2);
        assert_eq!(ff.badge, Some("Snap"));

        let tab = find(&gs, "cargo");
        assert_eq!(tab.members.len(), 3);
        assert_eq!(tab.badge, Some("Terminal"));

        let cups = find(&gs, "cups");
        assert_eq!(cups.section, Section::System);
        assert_eq!(cups.protection, Protection::Elevated);

        let init = gs.iter().find(|g| g.members.iter().any(|k| k.pid == 1)).unwrap();
        assert_eq!(init.protection, Protection::Blocked);
        // The user manager is a boundary: it must not swallow its children.
        let user_mgr = gs.iter().find(|g| g.members.iter().any(|k| k.pid == 50)).unwrap();
        assert_eq!(user_mgr.members.len(), 1);
        assert_eq!(user_mgr.protection, Protection::Blocked);
    }

    #[test]
    fn real_system_smoke() {
        let mut s = glance_proc::Scanner::new().unwrap();
        s.scan().unwrap();
        let mut g = Grouper::new(DesktopIndex::load(), s.own_uid());
        let gs = g.group(s.entries());
        let total: usize = gs.iter().map(|g| g.members.len()).sum();
        assert_eq!(total, s.entries().len(), "every process lands in exactly one group");
    }
}
