//! Index of installed `.desktop` application entries.
//!
//! Built once at startup by scanning the XDG application directories, then
//! queried by desktop ID, executable path, executable name or install
//! directory.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// One application entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    /// Desktop file ID, e.g. `org.gnome.Ptyxis.desktop`.
    pub id: Arc<str>,
    /// Localized display name.
    pub name: Arc<str>,
    pub icon: Option<Arc<str>>,
    /// First program in `Exec=` as written (absolute or bare name).
    pub exec: Option<String>,
    pub wm_class: Option<String>,
    pub no_display: bool,
    /// Where the entry was loaded from.
    pub path: PathBuf,
}

/// Lookup tables over all visible entries.
#[derive(Debug, Default)]
pub struct DesktopIndex {
    entries: Vec<DesktopEntry>,
    by_id: HashMap<String, usize>,
    by_exe: HashMap<PathBuf, usize>,
    by_name: HashMap<String, usize>,
    by_dir: HashMap<PathBuf, Option<usize>>,
}

/// Interpreters and launchers: never index these as an app's executable,
/// otherwise every Python script would match whichever app happened to
/// use `Exec=python3 ...`.
const GENERIC_PROGRAMS: &[&str] = &[
    "env", "sh", "bash", "dash", "zsh", "fish", "snap", "flatpak", "python", "python2", "python3",
    "perl", "ruby", "node", "nodejs", "java", "mono", "wine", "wine64", "gjs", "electron", "pkexec",
    "sudo", "gtk-launch", "xdg-open", "busctl", "gapplication", "dbus-send", "systemd-run",
];

/// Directories shared by many unrelated programs. Matching "same install
/// directory" is only meaningful for app-private directories like
/// `/opt/google/chrome` or `/usr/lib/firefox`.
const GENERIC_DIRS: &[&str] = &[
    "/", "/bin", "/sbin", "/usr", "/usr/bin", "/usr/sbin", "/usr/local", "/usr/local/bin",
    "/usr/local/sbin", "/usr/lib", "/usr/lib64", "/usr/lib32", "/usr/libexec", "/usr/share",
    "/usr/games", "/opt", "/snap", "/snap/bin", "/usr/lib/x86_64-linux-gnu",
    "/usr/lib/aarch64-linux-gnu", "/usr/libexec/x86_64-linux-gnu", "/var/lib/flatpak",
];

impl DesktopIndex {
    /// Load from the standard XDG application directories.
    pub fn load() -> Self {
        Self::load_dirs(&xdg_application_dirs())
    }

    /// Load from explicit `applications` directories, highest precedence
    /// first. Entries with an already-seen ID are ignored (XDG shadowing).
    pub fn load_dirs(dirs: &[PathBuf]) -> Self {
        let langs = locale_candidates();
        let path_env = std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".into());
        let mut idx = Self::default();
        let mut seen = std::collections::HashSet::new();
        for dir in dirs {
            let mut files = Vec::new();
            collect_desktop_files(dir, dir, 0, &mut files);
            for (id, path) in files {
                if !seen.insert(id.clone()) {
                    continue;
                }
                let Ok(text) = fs::read_to_string(&path) else { continue };
                if let Some(entry) = parse_entry(&id, &text, &path, &langs) {
                    idx.insert(entry, &path_env);
                }
            }
        }
        idx
    }

    /// Add an entry from `.desktop` file text (used by tests and fixtures).
    pub fn insert_text(&mut self, id: &str, text: &str) {
        if let Some(entry) = parse_entry(id, text, Path::new(id), &[]) {
            self.insert(entry, "/usr/local/bin:/usr/bin:/bin");
        }
    }

    fn insert(&mut self, entry: DesktopEntry, path_env: &str) {
        let i = self.entries.len();
        self.by_id.insert(normalize_id(&entry.id), i);

        if let Some(exec) = entry.exec.as_deref() {
            let base = basename(exec);
            if !is_generic_program(base) {
                self.insert_name(base, i, entry.no_display);
                let resolved = resolve_program(exec, path_env);
                if let Some(p) = &resolved {
                    self.by_exe.entry(p.clone()).or_insert(i);
                    if let Ok(real) = fs::canonicalize(p) {
                        let real_base = real.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !is_generic_program(real_base) {
                            self.insert_name(real_base, i, entry.no_display);
                            if !entry.no_display {
                                self.insert_dir(&real, i);
                            }
                            self.by_exe.entry(real).or_insert(i);
                        }
                    }
                }
            }
        }
        if let Some(wm) = entry.wm_class.as_deref() {
            self.insert_name(wm, i, entry.no_display);
        }
        self.entries.push(entry);
    }

    fn insert_name(&mut self, name: &str, i: usize, no_display: bool) {
        let key = name.to_ascii_lowercase();
        match self.by_name.get(&key) {
            // Prefer visible entries over NoDisplay helpers.
            Some(&old) if !(self.entry_hidden(old) && !no_display) => {}
            _ => {
                self.by_name.insert(key, i);
            }
        }
    }

    fn entry_hidden(&self, i: usize) -> bool {
        self.entries.get(i).is_some_and(|e| e.no_display)
    }

    fn insert_dir(&mut self, exe: &Path, i: usize) {
        if let Some(dir) = exe.parent()
            && !is_generic_dir(dir)
        {
            self.by_dir
                .entry(dir.to_path_buf())
                .and_modify(|v| {
                    if *v != Some(i) {
                        *v = None; // ambiguous: several apps share it
                    }
                })
                .or_insert(Some(i));
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[DesktopEntry] {
        &self.entries
    }

    /// Look up by desktop ID; `.desktop` suffix and case are optional.
    pub fn by_id(&self, id: &str) -> Option<&DesktopEntry> {
        self.by_id.get(&normalize_id(id)).map(|&i| &self.entries[i])
    }

    /// First entry whose ID starts with `prefix` (e.g. `firefox_` for snaps).
    pub fn by_id_prefix(&self, prefix: &str) -> Option<&DesktopEntry> {
        let p = prefix.to_ascii_lowercase();
        self.entries
            .iter()
            .filter(|e| e.id.to_ascii_lowercase().starts_with(&p))
            .min_by_key(|e| e.no_display)
    }

    /// Match an executable path: exact path first, then an app-private
    /// install directory (up to three levels up, e.g. helpers under
    /// `/usr/lib/app/resources/node/bin`).
    pub fn by_exe(&self, exe: &Path) -> Option<&DesktopEntry> {
        if let Some(&i) = self.by_exe.get(exe) {
            return Some(&self.entries[i]);
        }
        let mut dir = exe.parent();
        for _ in 0..4 {
            let d = dir?;
            if is_generic_dir(d) {
                return None;
            }
            if let Some(&slot) = self.by_dir.get(d) {
                return slot.map(|i| &self.entries[i]);
            }
            dir = d.parent();
        }
        None
    }

    /// Match a bare program name or WM class (case-insensitive).
    pub fn by_name(&self, name: &str) -> Option<&DesktopEntry> {
        self.by_name.get(&name.to_ascii_lowercase()).map(|&i| &self.entries[i])
    }
}

/// `python3.14` counts as `python3`.
fn is_generic_program(name: &str) -> bool {
    GENERIC_PROGRAMS.contains(&name)
        || GENERIC_PROGRAMS.contains(&name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.'))
}

fn is_generic_dir(d: &Path) -> bool {
    let s = d.to_str().unwrap_or("/");
    if GENERIC_DIRS.contains(&s) {
        return true;
    }
    // Per-user bin directories.
    s.ends_with("/.local/bin") || s.ends_with("/bin") && s.matches('/').count() <= 3
}

fn normalize_id(id: &str) -> String {
    id.strip_suffix(".desktop").unwrap_or(id).to_ascii_lowercase()
}

fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn resolve_program(prog: &str, path_env: &str) -> Option<PathBuf> {
    if prog.starts_with('/') {
        return Some(PathBuf::from(prog));
    }
    path_env
        .split(':')
        .filter(|d| !d.is_empty())
        .map(|d| Path::new(d).join(prog))
        .find(|p| p.is_file())
}

/// XDG application directories, highest precedence first, plus the Snap and
/// Flatpak export directories (in case Glance runs with a minimal env).
pub fn xdg_application_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.as_ref().map(|h| h.join(".local/share")));
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if !dirs.contains(&p) {
            dirs.push(p);
        }
    };
    if let Some(d) = data_home {
        push(d.join("applications"));
    }
    for d in data_dirs.split(':').filter(|d| d.starts_with('/')) {
        push(Path::new(d.trim_end_matches('/')).join("applications"));
    }
    if let Some(h) = &home {
        push(h.join(".local/share/flatpak/exports/share/applications"));
    }
    push("/var/lib/flatpak/exports/share/applications".into());
    push("/var/lib/snapd/desktop/applications".into());
    dirs
}

fn collect_desktop_files(root: &Path, dir: &Path, depth: u8, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut items: Vec<_> = rd.flatten().collect();
    items.sort_by_key(|e| e.file_name());
    for e in items {
        let path = e.path();
        let ft = match e.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if ft.is_dir() || (ft.is_symlink() && path.is_dir()) {
            if depth < 3 {
                collect_desktop_files(root, &path, depth + 1, out);
            }
        } else if path.extension().is_some_and(|x| x == "desktop") {
            // Desktop file ID: path relative to the applications dir with
            // '/' replaced by '-'.
            if let Ok(rel) = path.strip_prefix(root) {
                let id = rel.to_string_lossy().replace('/', "-");
                out.push((id, path));
            }
        }
    }
}

/// `LANG=pt_BR.UTF-8` gives `["pt_BR", "pt"]`.
fn locale_candidates() -> Vec<String> {
    let raw = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
        .unwrap_or_default();
    let base = raw.split(['.', '@']).next().unwrap_or("");
    let mut out = Vec::new();
    if base.is_empty() || base == "C" || base == "POSIX" {
        return out;
    }
    out.push(base.to_owned());
    if let Some((lang, _)) = base.split_once('_') {
        out.push(lang.to_owned());
    }
    out
}

fn parse_entry(id: &str, text: &str, path: &Path, langs: &[String]) -> Option<DesktopEntry> {
    let mut in_group = false;
    let mut kv: HashMap<&str, &str> = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            if in_group {
                break; // only [Desktop Entry] matters
            }
            in_group = line == "[Desktop Entry]";
            continue;
        }
        if in_group && let Some((k, v)) = line.split_once('=') {
            kv.entry(k.trim()).or_insert(v.trim());
        }
    }
    if kv.get("Type").copied() != Some("Application") {
        return None;
    }
    if kv.get("Hidden").copied() == Some("true") {
        return None;
    }
    let localized = |key: &str| -> Option<String> {
        langs
            .iter()
            .find_map(|l| kv.get(format!("{key}[{l}]").as_str()).copied())
            .or_else(|| kv.get(key).copied())
            .map(unescape_value)
    };
    let name = localized("Name").filter(|n| !n.is_empty())?;
    let exec = kv
        .get("TryExec")
        .map(|s| unescape_value(s))
        .filter(|s| !s.is_empty())
        .or_else(|| kv.get("Exec").and_then(|e| exec_program(e)));
    Some(DesktopEntry {
        id: id.into(),
        name: name.into(),
        icon: kv.get("Icon").map(|s| unescape_value(s)).filter(|s| !s.is_empty()).map(Into::into),
        exec,
        wm_class: kv.get("StartupWMClass").map(|s| unescape_value(s)).filter(|s| !s.is_empty()),
        no_display: kv.get("NoDisplay").copied() == Some("true"),
        path: path.to_path_buf(),
    })
}

/// The program `Exec=` actually runs: skips `env VAR=x` prefixes and
/// handles double quotes.
fn exec_program(exec: &str) -> Option<String> {
    let tokens = tokenize_exec(exec);
    let mut it = tokens.into_iter().peekable();
    while let Some(t) = it.next() {
        if t == "env" || t.ends_with("/env") {
            while it.peek().is_some_and(|n| n.contains('=') || n.starts_with('-')) {
                it.next();
            }
            continue;
        }
        if t.starts_with('%') {
            continue;
        }
        return Some(t);
    }
    None
}

fn tokenize_exec(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = s.chars();
    let mut any = false;
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                any = true;
            }
            '\\' if quoted => {
                if let Some(n) = chars.next() {
                    cur.push(n);
                }
            }
            ' ' | '\t' if !quoted => {
                if any || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    any = false;
                }
            }
            _ => cur.push(c),
        }
    }
    if any || !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn unescape_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('s') => out.push(' '),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "[Desktop Entry]\nName=Visual Studio Code\nExec=/usr/share/code/code %F\n\
        Icon=vscode\nType=Application\nStartupWMClass=Code\n\n[Desktop Action new-empty-window]\nName=New Window\nExec=/bin/false\n";

    #[test]
    fn parses_entry() {
        let e = parse_entry("code.desktop", CODE, Path::new("x"), &[]).unwrap();
        assert_eq!(&*e.name, "Visual Studio Code");
        assert_eq!(e.exec.as_deref(), Some("/usr/share/code/code"));
        assert_eq!(e.icon.as_deref(), Some("vscode"));
        assert_eq!(e.wm_class.as_deref(), Some("Code"));
        assert!(!e.no_display);
    }

    #[test]
    fn localized_name() {
        let t = "[Desktop Entry]\nType=Application\nName=Files\nName[pt_BR]=Arquivos\nName[pt]=Ficheiros\nExec=nautilus\n";
        let e = parse_entry("n.desktop", t, Path::new("x"), &["pt_BR".into(), "pt".into()]).unwrap();
        assert_eq!(&*e.name, "Arquivos");
        let e = parse_entry("n.desktop", t, Path::new("x"), &["pt_PT".into(), "pt".into()]).unwrap();
        assert_eq!(&*e.name, "Ficheiros");
    }

    #[test]
    fn exec_skips_env_and_quotes() {
        assert_eq!(
            exec_program("env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/firefox_firefox.desktop /snap/bin/firefox %u").as_deref(),
            Some("/snap/bin/firefox")
        );
        assert_eq!(exec_program("\"/opt/My App/app\" --x").as_deref(), Some("/opt/My App/app"));
    }

    #[test]
    fn rejects_non_apps() {
        assert!(parse_entry("l.desktop", "[Desktop Entry]\nType=Link\nName=x\n", Path::new("x"), &[]).is_none());
        assert!(parse_entry("h.desktop", "[Desktop Entry]\nType=Application\nName=x\nHidden=true\n", Path::new("x"), &[]).is_none());
    }

    #[test]
    fn lookups() {
        let mut idx = DesktopIndex::default();
        idx.insert_text("code.desktop", CODE);
        idx.insert_text(
            "py.desktop",
            "[Desktop Entry]\nType=Application\nName=Py Tool\nExec=python3 /usr/bin/pytool\n",
        );
        assert_eq!(&*idx.by_id("CODE").unwrap().name, "Visual Studio Code");
        assert_eq!(&*idx.by_id("code.desktop").unwrap().name, "Visual Studio Code");
        assert_eq!(&*idx.by_name("code").unwrap().name, "Visual Studio Code");
        assert_eq!(&*idx.by_exe(Path::new("/usr/share/code/code")).unwrap().name, "Visual Studio Code");
        assert!(idx.by_name("python3").is_none(), "interpreters are never indexed");
        assert!(is_generic_program("python3.14"));
    }

    #[test]
    fn generic_dirs() {
        assert!(is_generic_dir(Path::new("/usr/bin")));
        assert!(is_generic_dir(Path::new("/home/u/.local/bin")));
        assert!(!is_generic_dir(Path::new("/opt/google/chrome")));
        assert!(!is_generic_dir(Path::new("/usr/lib/firefox")));
    }
}
