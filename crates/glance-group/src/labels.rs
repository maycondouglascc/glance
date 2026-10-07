//! Friendly names for individual processes.
//!
//! Multi-process apps (Chromium, Electron, Firefox) run many processes with
//! the same executable. Their role is encoded in argv; this module turns it
//! into a short label like "Renderer" or "GPU Process".

use glance_proc::ProcStatic;

const INTERPRETERS: &[&str] = &[
    "python", "python2", "python3", "node", "nodejs", "perl", "ruby", "bash", "sh", "dash", "zsh",
    "gjs", "java", "deno", "bun",
];

/// Short display label for one process.
pub fn process_label(info: &ProcStatic) -> String {
    if info.is_kernel_thread {
        return info.comm.to_string();
    }
    let cmd = String::from_utf8_lossy(&info.cmdline);
    // Chromium may rewrite argv into one space-joined string, so search the
    // whole command line rather than individual arguments.
    let cmd = cmd.replace('\0', " ");

    if let Some(t) = flag_value(&cmd, "--type=") {
        return chromium_label(t, &cmd);
    }
    if info.exe_name().contains("crashpad_handler") || info.comm.contains("crashpad") {
        return "Crash Handler".into();
    }
    if cmd.contains(" -contentproc") {
        return firefox_label(info, &cmd);
    }

    let argv0 = info.argv().next().map(|a| String::from_utf8_lossy(a).into_owned());
    let argv0_name = argv0
        .as_deref()
        .map(|a| a.split(' ').next().unwrap_or(a))
        .map(|a| a.rsplit('/').next().unwrap_or(a).to_owned());

    let exe = info.exe_name();
    if is_interpreter(exe) || argv0_name.as_deref().is_some_and(is_interpreter) {
        if let Some(script) = script_name(info) {
            return script;
        }
    }

    // comm is truncated to 15 bytes; prefer argv0 when comm is its prefix.
    match argv0_name {
        Some(a) if a.len() > info.comm.len() && a.starts_with(&*info.comm) => a,
        _ => info.comm.to_string(),
    }
}

fn is_interpreter(name: &str) -> bool {
    let base = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    INTERPRETERS.contains(&name) || INTERPRETERS.contains(&base)
}

/// First non-option argument after the interpreter (or `-m module`).
pub fn script_name(info: &ProcStatic) -> Option<String> {
    let mut args = info.argv().skip(1).map(|a| String::from_utf8_lossy(a).into_owned());
    let java = info.exe_name() == "java" || info.comm.as_ref() == "java";
    while let Some(a) = args.next() {
        if java {
            match a.as_str() {
                "-jar" => return args.next().map(|j| j.rsplit('/').next().unwrap_or(&j).to_owned()),
                "-cp" | "-classpath" | "--class-path" | "-p" | "--module-path" | "--add-opens"
                | "--add-exports" | "--add-reads" | "--add-modules" | "--patch-module" => {
                    args.next();
                    continue;
                }
                "-m" | "--module" => {
                    return args.next().map(|m| m.rsplit(['/', '.']).next().unwrap_or(&m).to_owned());
                }
                s if s.starts_with('-') => continue,
                // Main class: keep the simple name.
                s => return Some(s.rsplit('.').next().unwrap_or(s).to_owned()),
            }
        }
        if a == "-m" {
            return args.next();
        }
        if a == "-c" || a == "-e" {
            return None;
        }
        if a.starts_with('-') {
            continue;
        }
        return Some(a.rsplit('/').next().unwrap_or(&a).to_owned());
    }
    None
}

fn flag_value<'a>(cmd: &'a str, flag: &str) -> Option<&'a str> {
    let i = cmd.find(flag)?;
    let rest = &cmd[i + flag.len()..];
    Some(rest.split(' ').next().unwrap_or(rest))
}

fn chromium_label(t: &str, cmd: &str) -> String {
    match t {
        "renderer" if cmd.contains("--extension-process") => "Extension".into(),
        "renderer" => "Renderer".into(),
        "gpu-process" => "GPU Process".into(),
        "zygote" => "Zygote".into(),
        "broker" => "Broker".into(),
        "crashpad-handler" => "Crash Handler".into(),
        "utility" => match flag_value(cmd, "--utility-sub-type=") {
            Some(s) if s.starts_with("network.") => "Network Service".into(),
            Some(s) if s.starts_with("storage.") => "Storage Service".into(),
            Some(s) if s.starts_with("audio.") => "Audio Service".into(),
            Some(s) if s.starts_with("video_capture.") => "Video Capture".into(),
            Some(s) if s.starts_with("node.") => "Node Service".into(),
            Some(s) if s.starts_with("data_decoder.") => "Data Decoder".into(),
            _ => "Utility".into(),
        },
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect::<String>().replace('-', " "),
                None => "Helper".into(),
            }
        }
    }
}

fn firefox_label(info: &ProcStatic, cmd: &str) -> String {
    // Firefox renames its children via prctl; undo the 15-byte truncation.
    let by_comm = match &*info.comm {
        "Isolated Web Co" => Some("Isolated Web Content"),
        "Privileged Cont" => Some("Privileged Content"),
        "Isolated Servic" => Some("Isolated Service Worker"),
        "Web Content" | "WebExtensions" | "RDD Process" | "Socket Process" | "Utility Process" => None,
        _ => None,
    };
    if let Some(l) = by_comm {
        return l.into();
    }
    match cmd.trim_end().rsplit(' ').next() {
        _ if info.comm.contains(' ') || info.comm.starts_with("Web") => info.comm.to_string(),
        Some("tab") => "Web Content".into(),
        Some("rdd") => "Media Decoder".into(),
        Some("socket") => "Network".into(),
        Some("gpu") => "GPU Process".into(),
        Some("utility") => "Utility".into(),
        Some("forkserver") => "Fork Server".into(),
        _ => info.comm.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glance_proc::LaunchHints;

    fn proc(comm: &str, exe: &str, argv: &[&str]) -> ProcStatic {
        ProcStatic {
            comm: comm.into(),
            cmdline: argv.join("\0").into_bytes().into(),
            exe: Some(std::path::Path::new(exe).into()),
            uid: Some(1000),
            cgroup: "".into(),
            hints: LaunchHints::default(),
            is_kernel_thread: false,
        }
    }

    #[test]
    fn chromium_roles() {
        let c = "/opt/google/chrome/chrome";
        assert_eq!(process_label(&proc("chrome", c, &[c, "--type=renderer", "--x"])), "Renderer");
        assert_eq!(
            process_label(&proc("chrome", c, &[c, "--type=renderer", "--extension-process"])),
            "Extension"
        );
        assert_eq!(process_label(&proc("chrome", c, &[c, "--type=gpu-process"])), "GPU Process");
        assert_eq!(
            process_label(&proc("chrome", c, &[c, "--type=utility", "--utility-sub-type=network.mojom.NetworkService"])),
            "Network Service"
        );
        // setproctitle-style single argv string
        assert_eq!(process_label(&proc("chrome", c, &["/opt/google/chrome/chrome --type=zygote"])), "Zygote");
        assert_eq!(process_label(&proc("chrome", c, &[c])), "chrome");
    }

    #[test]
    fn interpreters_and_truncation() {
        assert_eq!(
            process_label(&proc("python", "/usr/bin/python3.12", &["/usr/bin/python3", "-u", "/home/x/auto.py"])),
            "auto.py"
        );
        assert_eq!(process_label(&proc("python3", "/usr/bin/python3.12", &["python3", "-m", "http.server"])), "http.server");
        assert_eq!(
            process_label(&proc("gnome-session-b", "/usr/libexec/gnome-session-binary", &["/usr/libexec/gnome-session-binary"])),
            "gnome-session-binary"
        );
    }

    #[test]
    fn firefox_roles() {
        let f = "/snap/firefox/123/usr/lib/firefox/firefox";
        assert_eq!(
            process_label(&proc("Isolated Web Co", f, &[f, "-contentproc", "-isForBrowser", "tab"])),
            "Isolated Web Content"
        );
        assert_eq!(process_label(&proc("firefox", f, &[f, "-contentproc", "rdd"])), "Media Decoder");
    }
}
