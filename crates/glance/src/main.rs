//! Glance — Lightweight, high-performance Linux application & process monitor.

mod actions;
mod details;
mod dialogs;
mod icons;
mod row;
mod sampler;
mod settings;
mod tray;
mod window;

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use std::time::Duration;

use crate::sampler::start_sampler;
use crate::settings::Settings;
use crate::tray::{spawn_tray, TrayEvent};
use crate::window::MainWindow;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const APP_ID: &str = "io.github.maycon.Glance";
const CSS_DATA: &str = include_str!("style.css");

fn main() -> glib::ExitCode {
    // Default to Cairo renderer if not explicitly overridden by the user.
    // Cairo uses ~65MB RSS vs ~210MB for Vulkan/NGL, avoids loading libLLVM (40MB)
    // and Nvidia GPU compilation threads, while preserving 100% 60fps smoothness for 2D UI.
    if std::env::var_os("GSK_RENDERER").is_none() {
        unsafe {
            std::env::set_var("GSK_RENDERER", "cairo");
        }
    }

    let mock_path = std::env::args()
        .position(|a| a == "--snapshot-mock")
        .and_then(|idx| std::env::args().nth(idx + 1));

    if let Some(mock_out) = mock_path {
        let app = adw::Application::builder()
            .application_id("io.github.maycon.Glance.MockSnapshot")
            .build();
        app.connect_startup(|_| {
            load_css();
        });
        let mock_out_clone = mock_out.clone();
        app.connect_activate(move |application| {
            let (cmd_tx, _cmd_rx) = std::sync::mpsc::channel();
            let main_win = MainWindow::new(application, cmd_tx);
            main_win.window.present();

            let snapshot = create_mock_snapshot();
            main_win.apply_snapshot(snapshot);

            // Expand the second Firefox row
            main_win.expand_group(&glance_group::AppId::Desktop("firefox-exp".into()));

            // Re-apply snapshot to populate children
            let snap2 = create_mock_snapshot();
            main_win.apply_snapshot(snap2);

            let win = main_win.window.clone();
            let app_clone = application.clone();
            let out_file = mock_out_clone.clone();

            glib::timeout_add_local_once(Duration::from_millis(300), move || {
                let w = win.width();
                let h = win.height();
                let paintable = gtk::WidgetPaintable::new(Some(&win));
                let snapshot = gtk::Snapshot::new();
                gdk4::prelude::PaintableExt::snapshot(&paintable, &snapshot, f64::from(w), f64::from(h));
                if let Some(node) = snapshot.to_node() {
                    if let Some(renderer) = win.native().and_then(|n| n.renderer()) {
                        let texture = renderer.render_texture(node, None);
                        if let Err(e) = texture.save_to_png(&out_file) {
                            eprintln!("Failed to save snapshot: {}", e);
                        } else {
                            println!("Snapshot successfully saved to {}", out_file);
                        }
                    }
                }
                app_clone.quit();
            });
        });
        return app.run_with_args(&Vec::<String>::new());
    }

    Settings::load().sync_autostart();
    let start_hidden = std::env::args().any(|arg| arg == "--hidden");

    let app_id = std::env::var("GLANCE_APP_ID").unwrap_or_else(|_| APP_ID.to_string());
    let app = adw::Application::builder()
        .application_id(&app_id)
        .build();

    let main_window_holder: Rc<RefCell<Option<Rc<MainWindow>>>> = Rc::new(RefCell::new(None));

    // Spawn StatusNotifierItem tray in background
    let (tray_tx, tray_rx) = async_channel::unbounded::<TrayEvent>();
    let tray_handle = spawn_tray(tray_tx);

    app.connect_startup(|_| {
        load_css();
        Settings::load().sync_autostart();
    });

    // Handle tray events on GTK thread
    let holder_tray = main_window_holder.clone();
    let app_tray = app.clone();
    glib::spawn_future_local(async move {
        while let Ok(event) = tray_rx.recv().await {
            match event {
                TrayEvent::ToggleWindow => {
                    let holder = holder_tray.borrow();
                    if let Some(main_win) = holder.as_ref() {
                        let is_vis = main_win.window.is_visible();
                        if is_vis {
                            main_win.window.set_visible(false);
                        } else {
                            main_win.window.set_visible(true);
                            main_win.window.present();
                        }
                    }
                }
                TrayEvent::Quit => {
                    app_tray.quit();
                }
            }
        }
    });

    let holder_activate = main_window_holder.clone();
    let is_first_activate = Rc::new(RefCell::new(true));
    app.connect_activate(move |application| {
        let mut holder_borrow = holder_activate.borrow_mut();
        if let Some(main_win) = holder_borrow.as_ref() {
            main_win.window.set_visible(true);
            main_win.window.present();
            return;
        }

        // Initialize sampler thread
        let sampler = start_sampler();
        let cmd_tx = sampler.cmd_sender.clone();
        let snap_rx = sampler.snapshot_receiver;

        let main_win = MainWindow::new(application, cmd_tx.clone());
        *holder_borrow = Some(main_win.clone());

        // Stream snapshots to GTK thread
        let win_weak = Rc::downgrade(&main_win);
        glib::spawn_future_local(async move {
            while let Ok(snapshot) = snap_rx.recv().await {
                if let Some(win) = win_weak.upgrade() {
                    win.apply_snapshot(snapshot);
                } else {
                    break;
                }
            }
        });

        // Hide window on close request (Decision 2 from implementation plan)
        let win_close = main_win.window.clone();
        win_close.connect_close_request(move |win| {
            win.set_visible(false);
            glib::Propagation::Stop
        });

        let first = *is_first_activate.borrow();
        *is_first_activate.borrow_mut() = false;

        if !(first && start_hidden) {
            main_win.window.present();
        } else {
            let _ = cmd_tx.send(crate::sampler::SamplerCommand::Pause);
            #[cfg(target_os = "linux")]
            unsafe {
                libc::malloc_trim(0);
            }
        }
    });

    // Add global quit action
    let act_quit = gio::SimpleAction::new("quit", None);
    let app_quit = app.clone();
    act_quit.connect_activate(move |_, _| {
        app_quit.quit();
    });
    app.add_action(&act_quit);

    let clean_args: Vec<String> = std::env::args().filter(|a| a != "--hidden").collect();
    let exit_code = app.run_with_args(&clean_args);
    drop(tray_handle);
    exit_code
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_string(CSS_DATA);

    if let Some(display) = gdk4::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn create_mock_snapshot() -> crate::sampler::Snapshot {
    use glance_group::{AppGroup, AppId, Protection, Section};
    use glance_proc::{LaunchHints, ProcEntry, ProcKey, ProcSample, ProcStatic, SystemTotals};
    use std::collections::HashMap;
    use std::sync::Arc;

    let pids = [19220, 19221, 19222];
    let mut procs = HashMap::new();
    let mut members_exp = Vec::new();
    let mut members_col = Vec::new();

    for &pid in &pids {
        let key = ProcKey { pid, start_time: 100 };
        members_exp.push(key);
        let info = Arc::new(ProcStatic {
            comm: "Firefox".into(),
            cmdline: b"/usr/lib/firefox/firefox\0".to_vec().into_boxed_slice(),
            exe: Some(std::path::PathBuf::from("/usr/lib/firefox/firefox").into_boxed_path()),
            uid: Some(1000),
            cgroup: "/user.slice/user-1000.slice/app.slice/firefox.scope".into(),
            hints: LaunchHints::default(),
            is_kernel_thread: false,
        });
        let sample = ProcSample {
            ppid: 1,
            state: b'S',
            cpu_ticks: 100,
            rss_kib: 1171875, // 1.2 GB in decimal
            threads: 32,
        };
        let mut entry = ProcEntry::new(key, info, sample);
        entry.cpu_permille = Some(42); // 4.2%
        procs.insert(pid, entry);
    }

    for pid in 19223..=19227 {
        let key = ProcKey { pid, start_time: 100 };
        members_exp.push(key);
    }

    for pid in 19300..=19307 {
        let key = ProcKey { pid, start_time: 100 };
        members_col.push(key);
    }

    let groups = vec![
        AppGroup {
            id: AppId::Desktop("firefox-col".into()),
            section: Section::Apps,
            name: "Firefox".into(),
            icon: Some("firefox".into()),
            badge: None,
            members: members_col.clone(),
            roots: vec![members_col[0]],
            cpu_permille: 42,
            mem_kib: 1171875,
            mem_is_pss: true,
            protection: Protection::Normal,
            unit: None,
        },
        AppGroup {
            id: AppId::Desktop("firefox-exp".into()),
            section: Section::Apps,
            name: "Firefox".into(),
            icon: Some("firefox".into()),
            badge: None,
            members: members_exp.clone(),
            roots: vec![members_exp[0]],
            cpu_permille: 42,
            mem_kib: 1171875,
            mem_is_pss: true,
            protection: Protection::Normal,
            unit: None,
        },
    ];

    let totals = SystemTotals {
        cpu_permille: 131, // 13.1%
        mem_total_kib: 16 * 1024 * 1024,
        mem_available_kib: (16 * 1024 * 1024) - 2539062, // 2.6 GB in decimal
        ncpu: 8,
        process_count: 168,
    };

    crate::sampler::Snapshot {
        groups,
        procs: Arc::new(procs),
        totals,
        generation: 1,
        scan_time: Duration::from_millis(5),
        group_time: Duration::from_millis(2),
    }
}
