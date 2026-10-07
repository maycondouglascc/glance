//! Safety confirmation dialogs for process and application termination.

use adw::prelude::*;
use glance_group::Protection;
use glance_proc::signal::Sig;

use crate::settings::Settings;

/// Confirmation dialog specifically for terminating an entire application group.
pub fn confirm_app_termination<F>(
    parent: &impl IsA<gtk::Widget>,
    name: &str,
    protection: Protection,
    sig: Sig,
    on_confirm: F,
) where
    F: FnOnce() + 'static,
{
    match protection {
        Protection::Blocked => {
            let dialog = adw::AlertDialog::builder()
                .heading("Action Blocked")
                .body(format!(
                    "\"{}\" is a system critical component (e.g. PID 1, kernel thread, or session manager) and cannot be terminated.",
                    name
                ))
                .build();
            dialog.add_response("ok", "OK");
            dialog.set_default_response(Some("ok"));
            dialog.present(Some(parent));
        }
        Protection::Elevated => {
            let dialog = adw::AlertDialog::builder()
                .heading("Permission Denied")
                .body(format!(
                    "\"{}\" belongs to another user or system service. You do not have permissions to signal it.",
                    name
                ))
                .build();
            dialog.add_response("ok", "OK");
            dialog.set_default_response(Some("ok"));
            dialog.present(Some(parent));
        }
        Protection::Critical => {
            let dialog = adw::AlertDialog::builder()
                .heading("Critical Desktop Component")
                .body(format!(
                    "Warning: \"{}\" is an essential part of your desktop session. Terminating it may log you out immediately and cause unsaved work in all open applications to be lost.\n\nDo you want to proceed?",
                    name
                ))
                .build();
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("kill", if sig == Sig::Kill { "Force Kill" } else { "End" });
            dialog.set_response_appearance("kill", adw::ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));
            dialog.set_close_response("cancel");

            let on_confirm_cell = std::cell::RefCell::new(Some(on_confirm));
            dialog.connect_response(None, move |_, response| {
                if response == "kill" {
                    if let Some(cb) = on_confirm_cell.borrow_mut().take() {
                        cb();
                    }
                }
            });
            dialog.present(Some(parent));
        }
        Protection::Normal => {
            if sig == Sig::Kill {
                let dialog = adw::AlertDialog::builder()
                    .heading("Force Kill Application?")
                    .body(format!(
                        "Force killing \"{}\" will stop execution immediately with SIGKILL. Any unsaved data will be lost.",
                        name
                    ))
                    .build();
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("kill", "Force Kill");
                dialog.set_response_appearance("kill", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let on_confirm_cell = std::cell::RefCell::new(Some(on_confirm));
                dialog.connect_response(None, move |_, response| {
                    if response == "kill" {
                        if let Some(cb) = on_confirm_cell.borrow_mut().take() {
                            cb();
                        }
                    }
                });
                dialog.present(Some(parent));
            } else {
                // Graceful SIGTERM for normal application
                let settings = Settings::load();
                if settings.confirm_quit_app {
                    let dialog = adw::AlertDialog::builder()
                        .heading("Quit Application?")
                        .body(format!(
                            "Are you sure you want to quit \"{}\"? All associated processes will be terminated.",
                            name
                        ))
                        .build();

                    let check_box = gtk::CheckButton::builder()
                        .label("Não exibir novamente")
                        .margin_top(8)
                        .halign(gtk::Align::Center)
                        .build();

                    dialog.set_extra_child(Some(&check_box));
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("quit", "Quit Application");
                    dialog.set_response_appearance("quit", adw::ResponseAppearance::Destructive);
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");

                    let on_confirm_cell = std::cell::RefCell::new(Some(on_confirm));
                    dialog.connect_response(None, move |_, response| {
                        if response == "quit" {
                            if check_box.is_active() {
                                let mut s = Settings::load();
                                s.confirm_quit_app = false;
                                s.save();
                            }
                            if let Some(cb) = on_confirm_cell.borrow_mut().take() {
                                cb();
                            }
                        }
                    });
                    dialog.present(Some(parent));
                } else {
                    on_confirm();
                }
            }
        }
    }
}

/// Confirmation dialog for individual process termination.
pub fn confirm_termination<F>(
    parent: &impl IsA<gtk::Widget>,
    name: &str,
    protection: Protection,
    sig: Sig,
    on_confirm: F,
) where
    F: FnOnce() + 'static,
{
    match protection {
        Protection::Blocked => {
            let dialog = adw::AlertDialog::builder()
                .heading("Action Blocked")
                .body(format!(
                    "\"{}\" is a system critical component (e.g. PID 1, kernel thread, or session manager) and cannot be terminated.",
                    name
                ))
                .build();
            dialog.add_response("ok", "OK");
            dialog.set_default_response(Some("ok"));
            dialog.present(Some(parent));
        }
        Protection::Elevated => {
            let dialog = adw::AlertDialog::builder()
                .heading("Permission Denied")
                .body(format!(
                    "\"{}\" belongs to another user or system service. You do not have permissions to signal it.",
                    name
                ))
                .build();
            dialog.add_response("ok", "OK");
            dialog.set_default_response(Some("ok"));
            dialog.present(Some(parent));
        }
        Protection::Critical => {
            let dialog = adw::AlertDialog::builder()
                .heading("Critical Desktop Component")
                .body(format!(
                    "Warning: \"{}\" is an essential part of your desktop session. Terminating it may log you out immediately and cause unsaved work in all open applications to be lost.\n\nDo you want to proceed?",
                    name
                ))
                .build();
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("kill", if sig == Sig::Kill { "Force Kill" } else { "End" });
            dialog.set_response_appearance("kill", adw::ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));
            dialog.set_close_response("cancel");

            let on_confirm_cell = std::cell::RefCell::new(Some(on_confirm));
            dialog.connect_response(None, move |_, response| {
                if response == "kill" {
                    if let Some(cb) = on_confirm_cell.borrow_mut().take() {
                        cb();
                    }
                }
            });
            dialog.present(Some(parent));
        }
        Protection::Normal => {
            if sig == Sig::Kill {
                let dialog = adw::AlertDialog::builder()
                    .heading("Force Kill Process?")
                    .body(format!(
                        "Force killing \"{}\" will stop execution immediately with SIGKILL. Any unsaved data will be lost.",
                        name
                    ))
                    .build();
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("kill", "Force Kill");
                dialog.set_response_appearance("kill", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let on_confirm_cell = std::cell::RefCell::new(Some(on_confirm));
                dialog.connect_response(None, move |_, response| {
                    if response == "kill" {
                        if let Some(cb) = on_confirm_cell.borrow_mut().take() {
                            cb();
                        }
                    }
                });
                dialog.present(Some(parent));
            } else {
                // Graceful SIGTERM for normal processes proceeds directly without interruption
                on_confirm();
            }
        }
    }
}
