//! Details dialog for deep inspection of individual processes.

use adw::prelude::*;
use glance_group::format;
use glance_proc::ProcEntry;

pub fn show_process_details(parent: &impl IsA<gtk::Widget>, entry: &ProcEntry, label: &str) {
    let window = adw::Window::builder()
        .title(format!("Process Details — {} ({})", label, entry.key.pid))
        .modal(true)
        .transient_for(parent.root().and_downcast_ref::<gtk::Window>().unwrap())
        .default_width(480)
        .default_height(520)
        .build();

    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    group.set_title("Process Information");

    // PID & PPID
    let row_pid = adw::ActionRow::builder()
        .title("Process ID (PID)")
        .subtitle(format!("{}", entry.key.pid))
        .build();
    let row_ppid = adw::ActionRow::builder()
        .title("Parent PID (PPID)")
        .subtitle(format!("{}", entry.sample.ppid))
        .build();

    // Executable path
    let exe_str = entry
        .info
        .exe
        .as_deref()
        .and_then(|p| p.to_str())
        .unwrap_or("—");
    let row_exe = adw::ActionRow::builder()
        .title("Executable")
        .subtitle(exe_str)
        .build();

    // Copy Exe button
    let copy_exe_btn = gtk::Button::from_icon_name("edit-copy-symbolic");
    copy_exe_btn.set_valign(gtk::Align::Center);
    copy_exe_btn.set_tooltip_text(Some("Copy executable path"));
    let exe_copy = exe_str.to_string();
    copy_exe_btn.connect_clicked(move |btn| {
        btn.display().clipboard().set_text(&exe_copy);
    });
    row_exe.add_suffix(&copy_exe_btn);

    // State & Threads
    let state_desc = match entry.sample.state {
        b'R' => "Running (R)",
        b'S' => "Sleeping (S)",
        b'D' => "Disk Sleep / Uninterruptible (D)",
        b'Z' => "Zombie (Z)",
        b'T' => "Stopped (T)",
        b'I' => "Idle (I)",
        _ => "Unknown",
    };
    let row_state = adw::ActionRow::builder()
        .title("State")
        .subtitle(format!("{} · {} threads", state_desc, entry.sample.threads))
        .build();

    // Memory Breakdown
    let mem_sub = if let Some(pss) = entry.pss_kib {
        format!("PSS: {} · RSS: {}", format::mem(pss), format::mem(entry.sample.rss_kib))
    } else {
        format!("RSS: {}", format::mem(entry.sample.rss_kib))
    };
    let row_mem = adw::ActionRow::builder()
        .title("Memory Usage")
        .subtitle(mem_sub)
        .build();

    // CPU Usage
    let cpu_sub = entry
        .cpu_permille
        .map_or("—".to_string(), |c| format::cpu(u32::from(c)));
    let row_cpu = adw::ActionRow::builder()
        .title("CPU Usage (Total Machine)")
        .subtitle(cpu_sub)
        .build();

    // User / UID
    let uid_sub = entry
        .info
        .uid
        .map_or("—".to_string(), |u| format!("UID {}", u));
    let row_user = adw::ActionRow::builder()
        .title("User")
        .subtitle(uid_sub)
        .build();

    // cgroup
    let row_cgroup = adw::ActionRow::builder()
        .title("Control Group (cgroup)")
        .subtitle(if entry.info.cgroup.is_empty() { "—" } else { &entry.info.cgroup })
        .build();

    group.add(&row_pid);
    group.add(&row_ppid);
    group.add(&row_exe);
    group.add(&row_state);
    group.add(&row_cpu);
    group.add(&row_mem);
    group.add(&row_user);
    group.add(&row_cgroup);
    page.add(&group);

    // Command line group
    let cmd_group = adw::PreferencesGroup::new();
    cmd_group.set_title("Command Line");
    let cmdline_str = entry.info.display_cmdline();
    let cmd_entry = gtk::TextView::builder()
        .editable(false)
        .wrap_mode(gtk::WrapMode::Char)
        .monospace(true)
        .left_margin(8)
        .right_margin(8)
        .top_margin(8)
        .bottom_margin(8)
        .build();
    cmd_entry.buffer().set_text(&cmdline_str);
    cmd_entry.add_css_class("card");

    let copy_cmd_btn = gtk::Button::builder()
        .label("Copy Command Line")
        .icon_name("edit-copy-symbolic")
        .halign(gtk::Align::End)
        .margin_top(8)
        .build();
    let cmd_copy = cmdline_str.clone();
    copy_cmd_btn.connect_clicked(move |btn| {
        btn.display().clipboard().set_text(&cmd_copy);
    });

    let cmd_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    cmd_box.append(&cmd_entry);
    cmd_box.append(&copy_cmd_btn);
    cmd_group.add(&cmd_box);
    page.add(&cmd_group);

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&page));

    window.set_content(Some(&toolbar_view));
    window.present();
}
