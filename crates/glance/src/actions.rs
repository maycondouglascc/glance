use glance_group::AppGroup;
use glance_proc::signal::{send_signal, Sig, SignalError};
use glance_proc::ProcKey;

pub enum TerminateOutcome {
    Success,
    AlreadyExited,
    PermissionDenied,
    Error(String),
}

/// Terminate an individual process.
pub fn kill_proc(key: ProcKey, sig: Sig) -> TerminateOutcome {
    match send_signal(key, sig) {
        Ok(()) => TerminateOutcome::Success,
        Err(SignalError::Gone) => TerminateOutcome::AlreadyExited,
        Err(SignalError::PermissionDenied) => TerminateOutcome::PermissionDenied,
        Err(SignalError::Os(e)) => TerminateOutcome::Error(e.to_string()),
    }
}

/// Terminate an entire application group.
///
/// For graceful termination (`Sig::Term`), signals roots first so the app
/// can close its own child processes cleanly.
/// For force kill (`Sig::Kill`), signals all members.
pub fn kill_group(group: &AppGroup, sig: Sig) -> TerminateOutcome {
    // If unit is tracked, also attempt systemd KillUnit via systemctl for clean group reaping
    if let Some(unit) = &group.unit {
        let sig_name = match sig {
            Sig::Term => "SIGTERM",
            Sig::Kill => "SIGKILL",
        };
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "kill", "--kill-who=all", &format!("--signal={sig_name}"), unit])
            .output();
    }

    let targets = match sig {
        Sig::Term => {
            if group.roots.is_empty() {
                &group.members
            } else {
                &group.roots
            }
        }
        Sig::Kill => &group.members,
    };

    let mut any_success = false;
    for &key in targets {
        if let Ok(()) = send_signal(key, sig) {
            any_success = true;
        }
    }

    if any_success {
        TerminateOutcome::Success
    } else {
        TerminateOutcome::AlreadyExited
    }
}

/// Helper to format the outcome for toast/dialog display.
pub fn outcome_message(name: &str, sig: Sig, outcome: TerminateOutcome) -> String {
    let sig_label = match sig {
        Sig::Term => "Ended",
        Sig::Kill => "Force killed",
    };
    match outcome {
        TerminateOutcome::Success => format!("{sig_label} {name}"),
        TerminateOutcome::AlreadyExited => format!("{name} has already exited"),
        TerminateOutcome::PermissionDenied => format!("Permission denied for {name}"),
        TerminateOutcome::Error(e) => format!("Failed to signal {name}: {e}"),
    }
}
