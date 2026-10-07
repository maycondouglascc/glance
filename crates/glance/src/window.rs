#![allow(dead_code)]
//! Main application window with progressive disclosure list, search, sorting and footer totals.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use glance_group::format;
use glance_group::{AppGroup, AppId, Section};
use gtk::glib;

use crate::row::{AppRowWidget, ToastCallback};
use crate::sampler::{SamplerCommand, Snapshot};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortBy {
    Cpu,
    Memory,
    Count,
    Name,
}

pub struct MainWindow {
    pub window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    apps_container: gtk::Box,
    background_container: gtk::Box,
    system_container: gtk::Box,
    kernel_container: gtk::Box,
    bg_revealer: gtk::Revealer,
    bg_expand_btn: gtk::Button,
    sys_revealer: gtk::Revealer,
    sys_expand_btn: gtk::Button,
    kern_revealer: gtk::Revealer,
    kern_expand_btn: gtk::Button,
    kern_section_box: gtk::Box,
    footer_label: gtk::Label,
    search_entry: gtk::SearchEntry,
    sort_by: Rc<RefCell<SortBy>>,
    show_kernel: Rc<RefCell<bool>>,
    is_hovered: Rc<RefCell<bool>>,
    app_rows: Rc<RefCell<HashMap<AppId, AppRowWidget>>>,
    last_snapshot: Rc<RefCell<Option<Snapshot>>>,
    sampler_tx: std::sync::mpsc::Sender<SamplerCommand>,
    last_footer: RefCell<String>,
}

impl MainWindow {
    pub fn new(
        app: &adw::Application,
        sampler_tx: std::sync::mpsc::Sender<SamplerCommand>,
    ) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Glance")
            .default_width(442)
            .default_height(602)
            .width_request(380)
            .height_request(420)
            .build();
        window.set_icon_name(Some("io.github.maycon.Glance"));
        window.add_css_class("glance-window");

        let toast_overlay = adw::ToastOverlay::new();
        let toolbar_view = adw::ToolbarView::new();

        // --- Header Bar ---
        let header_bar = adw::HeaderBar::new();
        header_bar.add_css_class("glance-header");
        header_bar.add_css_class("flat");
        header_bar.set_show_start_title_buttons(false);
        header_bar.set_show_end_title_buttons(false);
        header_bar.set_centering_policy(adw::CenteringPolicy::Strict);

        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Search apps or PID…")
            .width_request(260)
            .height_request(32)
            .hexpand(false)
            .halign(gtk::Align::Center)
            .build();
        search_entry.add_css_class("glance-search");

        let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        search_box.set_halign(gtk::Align::Center);
        search_box.set_valign(gtk::Align::Center);
        search_box.set_size_request(260, 32);
        search_box.append(&search_entry);
        header_bar.set_title_widget(Some(&search_box));

        // Sort Menu Button
        let sort_menu = gio::Menu::new();
        sort_menu.append(Some("Sort by CPU"), Some("win.sort_cpu"));
        sort_menu.append(Some("Sort by Memory"), Some("win.sort_mem"));
        sort_menu.append(Some("Sort by Process Count"), Some("win.sort_count"));
        sort_menu.append(Some("Sort by Name"), Some("win.sort_name"));

        let sort_btn = gtk::MenuButton::builder()
            .menu_model(&sort_menu)
            .tooltip_text("Sort order")
            .build();
        sort_btn.set_child(Some(&crate::icons::sort_icon()));
        sort_btn.set_size_request(32, 32);
        sort_btn.add_css_class("header-icon-btn");
        sort_btn.add_css_class("circular");
        header_bar.pack_start(&sort_btn);

        // App Menu Button (3 dots)
        let app_menu = gio::Menu::new();
        app_menu.append(Some("Show Kernel Threads"), Some("win.toggle_kernel"));
        app_menu.append(Some("Start with System"), Some("win.toggle_autostart"));
        app_menu.append(Some("Confirm on Quit App"), Some("win.toggle_confirm_quit"));
        app_menu.append(Some("About Glance"), Some("win.about"));
        app_menu.append(Some("Quit"), Some("app.quit"));

        let menu_btn = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .menu_model(&app_menu)
            .tooltip_text("Main menu")
            .build();
        menu_btn.set_size_request(32, 32);
        menu_btn.add_css_class("header-icon-btn");
        menu_btn.add_css_class("circular");

        // Close Panel Button
        let close_btn = gtk::Button::builder()
            .icon_name("window-close-symbolic")
            .tooltip_text("Close Glance")
            .build();
        close_btn.set_size_request(32, 32);
        close_btn.add_css_class("header-icon-btn");
        close_btn.add_css_class("circular");
        let win_for_close = window.clone();
        close_btn.connect_clicked(move |_| {
            win_for_close.close();
        });

        header_bar.pack_end(&close_btn);
        header_bar.pack_end(&menu_btn);

        toolbar_view.add_top_bar(&header_bar);

        // --- Sticky Column Header Bar ---
        let col_header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        col_header.set_margin_top(4);
        col_header.set_margin_bottom(6);
        col_header.set_margin_start(16);
        col_header.set_margin_end(16);
        col_header.add_css_class("column-header");

        let col_name = gtk::Label::builder()
            .label("APPLICATION")
            .halign(gtk::Align::Start)
            .build();
        col_name.add_css_class("col-header-app");

        let col_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        col_spacer.set_hexpand(true);

        let col_metrics = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let col_cpu = gtk::Label::builder()
            .label("CPU")
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        col_cpu.set_width_request(36);
        col_cpu.add_css_class("col-header-metric");

        let col_mem = gtk::Label::builder()
            .label("RAM")
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        col_mem.set_width_request(38);
        col_mem.set_margin_start(24);
        col_mem.add_css_class("col-header-metric");

        let col_actions_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        col_actions_spacer.set_width_request(44);
        col_actions_spacer.set_margin_start(16);

        col_metrics.append(&col_cpu);
        col_metrics.append(&col_mem);
        col_metrics.append(&col_actions_spacer);

        col_header.append(&col_name);
        col_header.append(&col_spacer);
        col_header.append(&col_metrics);

        // --- Scrolled List Content ---
        let scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        list_box.set_margin_start(12);
        list_box.set_margin_end(12);
        list_box.set_margin_top(2);
        list_box.set_margin_bottom(6);

        // APPS Section
        let apps_container = gtk::Box::new(gtk::Orientation::Vertical, 2);

        // BACKGROUND Section (Collapsible)
        let (bg_section_box, bg_expand_btn, bg_revealer, background_container) =
            Self::create_collapsible_section("BACKGROUND");

        // SYSTEM Section (Collapsible)
        let (sys_section_box, sys_expand_btn, sys_revealer, system_container) =
            Self::create_collapsible_section("SYSTEM");

        // KERNEL Section (Collapsible & hidden by default)
        let (kern_section_box, kern_expand_btn, kern_revealer, kernel_container) =
            Self::create_collapsible_section("KERNEL THREADS");
        kern_section_box.set_visible(false);

        list_box.append(&apps_container);
        list_box.append(&bg_section_box);
        list_box.append(&sys_section_box);
        list_box.append(&kern_section_box);
        scroll.set_child(Some(&list_box));

        // --- Bottom Status Bar (Footer) ---
        let footer_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        footer_box.set_margin_start(16);
        footer_box.set_margin_end(16);
        footer_box.add_css_class("footer-bar");

        let footer_label = gtk::Label::builder()
            .label("CPU 0.0%  ·  RAM 0 MB  ·  0 processes")
            .halign(gtk::Align::End)
            .hexpand(true)
            .build();
        footer_label.add_css_class("numeric");
        footer_label.add_css_class("footer-text");
        footer_box.append(&footer_label);

        let main_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_vbox.append(&col_header);
        main_vbox.append(&scroll);
        main_vbox.append(&footer_box);

        toolbar_view.set_content(Some(&main_vbox));
        toast_overlay.set_child(Some(&toolbar_view));
        window.set_content(Some(&toast_overlay));

        // Hover tracking to freeze sorting during cursor interaction
        let is_hovered = Rc::new(RefCell::new(false));
        let hover_controller = gtk::EventControllerMotion::new();
        let is_hovered_clone = is_hovered.clone();
        hover_controller.connect_enter(move |_, _, _| {
            *is_hovered_clone.borrow_mut() = true;
        });
        let is_hovered_clone2 = is_hovered.clone();
        hover_controller.connect_leave(move |_| {
            *is_hovered_clone2.borrow_mut() = false;
        });
        scroll.add_controller(hover_controller);

        let main_window = Rc::new(Self {
            window,
            toast_overlay,
            apps_container,
            background_container,
            system_container,
            kernel_container,
            bg_revealer,
            bg_expand_btn,
            sys_revealer,
            sys_expand_btn,
            kern_revealer,
            kern_expand_btn,
            kern_section_box,
            footer_label,
            search_entry,
            sort_by: Rc::new(RefCell::new(SortBy::Cpu)),
            show_kernel: Rc::new(RefCell::new(false)),
            is_hovered,
            app_rows: Rc::new(RefCell::new(HashMap::new())),
            last_snapshot: Rc::new(RefCell::new(None)),
            sampler_tx,
            last_footer: RefCell::new(String::new()),
        });

        main_window.setup_actions(app);
        main_window.setup_cadence_handlers();
        main_window.setup_revealer_listeners();

        main_window
    }

    fn create_collapsible_section(
        title: &str,
    ) -> (gtk::Box, gtk::Button, gtk::Revealer, gtk::Box) {
        let section_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        section_box.set_margin_top(16);

        let header_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        header_row.set_margin_start(4);
        header_row.set_margin_end(4);
        header_row.set_margin_top(0);
        header_row.set_margin_bottom(0);
        header_row.add_css_class("section-header");

        let expand_btn = gtk::Button::from_icon_name("pan-end-symbolic");
        expand_btn.add_css_class("chevron-btn");
        expand_btn.add_css_class("circular");

        let label = gtk::Label::builder()
            .label(title)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("section-title");

        header_row.append(&expand_btn);
        header_row.append(&label);

        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(150);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        revealer.set_child(Some(&content_box));

        // Toggle connection
        let rev_clone = revealer.clone();
        let btn_clone = expand_btn.clone();
        expand_btn.connect_clicked(move |_| {
            let revealed = rev_clone.reveals_child();
            rev_clone.set_reveal_child(!revealed);
            if !revealed {
                btn_clone.set_icon_name("pan-down-symbolic");
            } else {
                btn_clone.set_icon_name("pan-end-symbolic");
            }
        });

        // Single click on section header row also toggles
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(gdk4::BUTTON_PRIMARY);
        let rev_clone2 = revealer.clone();
        let btn_clone2 = expand_btn.clone();
        let expand_btn_weak = expand_btn.downgrade();
        let header_row_weak = header_row.downgrade();
        click_gesture.connect_pressed(move |_, n_press, x, y| {
            if n_press == 1 {
                if let (Some(hdr), Some(exp_b)) =
                    (header_row_weak.upgrade(), expand_btn_weak.upgrade())
                {
                    if let Some(picked) = hdr.pick(x, y, gtk::PickFlags::DEFAULT) {
                        if picked == exp_b || picked.is_ancestor(&exp_b) {
                            return;
                        }
                    }
                    let revealed = rev_clone2.reveals_child();
                    rev_clone2.set_reveal_child(!revealed);
                    if !revealed {
                        btn_clone2.set_icon_name("pan-down-symbolic");
                    } else {
                        btn_clone2.set_icon_name("pan-end-symbolic");
                    }
                }
            }
        });
        header_row.set_cursor_from_name(Some("pointer"));
        header_row.add_controller(click_gesture);

        section_box.append(&header_row);
        section_box.append(&revealer);

        (section_box, expand_btn, revealer, content_box)
    }

    fn setup_actions(self: &Rc<Self>, _app: &adw::Application) {
        let action_group = gio::SimpleActionGroup::new();

        // Sort actions
        let sorts = [
            ("sort_cpu", SortBy::Cpu),
            ("sort_mem", SortBy::Memory),
            ("sort_count", SortBy::Count),
            ("sort_name", SortBy::Name),
        ];

        for (action_name, sort_val) in sorts {
            let mw = self.clone();
            let action = gio::SimpleAction::new(action_name, None);
            action.connect_activate(move |_, _| {
                *mw.sort_by.borrow_mut() = sort_val;
                mw.request_relayout();
            });
            action_group.add_action(&action);
        }

        // Toggle Kernel Threads
        let mw_kern = self.clone();
        let act_kernel = gio::SimpleAction::new("toggle_kernel", None);
        act_kernel.connect_activate(move |_, _| {
            let cur = *mw_kern.show_kernel.borrow();
            *mw_kern.show_kernel.borrow_mut() = !cur;
            mw_kern.kern_section_box.set_visible(!cur);
            mw_kern.request_relayout();
        });
        action_group.add_action(&act_kernel);

        // Autostart toggle action
        let settings = crate::settings::Settings::load();
        let act_autostart = gio::SimpleAction::new_stateful(
            "toggle_autostart",
            None,
            &glib::Variant::from(settings.autostart),
        );
        act_autostart.connect_activate(|action, _| {
            let state = action.state().unwrap().get::<bool>().unwrap();
            let new_state = !state;
            action.set_state(&glib::Variant::from(new_state));
            let mut s = crate::settings::Settings::load();
            s.autostart = new_state;
            s.save();
            s.sync_autostart();
        });
        action_group.add_action(&act_autostart);

        // Confirm on Quit App action
        let act_confirm_quit = gio::SimpleAction::new_stateful(
            "toggle_confirm_quit",
            None,
            &glib::Variant::from(settings.confirm_quit_app),
        );
        act_confirm_quit.connect_activate(|action, _| {
            let state = action.state().unwrap().get::<bool>().unwrap();
            let new_state = !state;
            action.set_state(&glib::Variant::from(new_state));
            let mut s = crate::settings::Settings::load();
            s.confirm_quit_app = new_state;
            s.save();
        });
        action_group.add_action(&act_confirm_quit);

        // About Dialog
        let act_about = gio::SimpleAction::new("about", None);
        let win_clone = self.window.clone();
        act_about.connect_activate(move |_, _| {
            let about = adw::AboutDialog::builder()
                .application_name("Glance")
                .application_icon("io.github.maycon.Glance")
                .developer_name("Maycon & Antigravity")
                .version("0.1.0")
                .comments("Lightweight high-performance Linux process & application monitor.")
                .website("https://github.com/maycon/glance")
                .issue_url("https://github.com/maycon/glance/issues")
                .license_type(gtk::License::Gpl30Only)
                .build();
            about.present(Some(&win_clone));
        });
        action_group.add_action(&act_about);

        self.window.insert_action_group("win", Some(&action_group));

        // Connect search text change
        let mw_search = self.clone();
        self.search_entry.connect_search_changed(move |_| {
            mw_search.request_relayout();
        });
    }

    /// Manage adaptive refresh rates based on window focus & visibility.
    fn setup_cadence_handlers(&self) {
        let tx = self.sampler_tx.clone();
        self.window.connect_is_active_notify(move |win| {
            let is_active = win.is_active();
            let interval = if is_active {
                Duration::from_millis(1000)
            } else {
                Duration::from_millis(2000)
            };
            let _ = tx.send(SamplerCommand::SetInterval(interval));
        });

        let tx2 = self.sampler_tx.clone();
        self.window.connect_visible_notify(move |win| {
            if win.is_visible() {
                let _ = tx2.send(SamplerCommand::Resume);
                let _ = tx2.send(SamplerCommand::ImmediateScan);
            } else {
                let _ = tx2.send(SamplerCommand::Pause);
                #[cfg(target_os = "linux")]
                unsafe {
                    libc::malloc_trim(0);
                }
            }
        });
    }

    fn setup_revealer_listeners(self: &Rc<Self>) {
        let mw_bg = self.clone();
        self.bg_revealer.connect_reveal_child_notify(move |rev| {
            if rev.reveals_child() {
                mw_bg.request_relayout();
            } else {
                #[cfg(target_os = "linux")]
                unsafe {
                    libc::malloc_trim(0);
                }
            }
        });

        let mw_sys = self.clone();
        self.sys_revealer.connect_reveal_child_notify(move |rev| {
            if rev.reveals_child() {
                mw_sys.request_relayout();
            } else {
                #[cfg(target_os = "linux")]
                unsafe {
                    libc::malloc_trim(0);
                }
            }
        });

        let mw_kern = self.clone();
        self.kern_revealer.connect_reveal_child_notify(move |rev| {
            if rev.reveals_child() {
                mw_kern.request_relayout();
            } else {
                #[cfg(target_os = "linux")]
                unsafe {
                    libc::malloc_trim(0);
                }
            }
        });
    }

    pub fn toast(&self, text: &str) {
        let toast = adw::Toast::new(text);
        self.toast_overlay.add_toast(toast);
    }

    pub fn expand_group(&self, id: &AppId) {
        if let Some(row) = self.app_rows.borrow().get(id) {
            row.set_expanded(true);
        }
    }

    pub fn request_relayout(&self) {
        if let Some(snap) = self.last_snapshot.borrow().as_ref() {
            Self::relayout_groups(
                snap,
                *self.sort_by.borrow(),
                &self.search_entry.text(),
                *self.show_kernel.borrow(),
                &self.app_rows,
                &self.apps_container,
                &self.background_container,
                &self.system_container,
                &self.kernel_container,
                self.bg_revealer.reveals_child(),
                self.sys_revealer.reveals_child(),
                self.kern_revealer.reveals_child(),
                &self.toast_overlay,
            );
        }
    }

    /// Apply a snapshot from the sampler thread.
    pub fn apply_snapshot(&self, snapshot: Snapshot) {
        if !self.window.is_visible() {
            *self.last_snapshot.borrow_mut() = Some(snapshot);
            return;
        }

        // Update footer totals
        let tot = &snapshot.totals;
        let used_mem = tot.mem_total_kib.saturating_sub(tot.mem_available_kib);
        let footer_str = format!(
            "CPU {:.1}%  ·  RAM {}  ·  {} processes",
            tot.cpu_permille as f64 / 10.0,
            format::mem(used_mem),
            tot.process_count
        );
        let mut last_f = self.last_footer.borrow_mut();
        if *last_f != footer_str {
            *last_f = footer_str.clone();
            self.footer_label.set_text(&footer_str);
        }
        drop(last_f);

        let q = self.search_entry.text();
        let bg_rev = self.bg_revealer.reveals_child();
        let sys_rev = self.sys_revealer.reveals_child();
        let kern_rev = self.kern_revealer.reveals_child();

        // Synchronize row data
        let toast_fn: ToastCallback = {
            let toast_overlay = self.toast_overlay.clone();
            Rc::new(move |msg: &str| {
                toast_overlay.add_toast(adw::Toast::new(msg));
            })
        };

        {
            let mut app_rows = self.app_rows.borrow_mut();
            for group in &snapshot.groups {
                let is_needed = match group.section {
                    Section::Apps => true,
                    Section::Background => bg_rev || !q.is_empty(),
                    Section::System => sys_rev || !q.is_empty(),
                    Section::Kernel => *self.show_kernel.borrow() && (kern_rev || !q.is_empty()),
                };

                if !is_needed {
                    continue;
                }

                if let Some(row_widget) = app_rows.get_mut(&group.id) {
                    row_widget.update(group, &snapshot.procs);
                } else {
                    let mut row_widget = AppRowWidget::new(group.clone(), toast_fn.clone());
                    row_widget.update(group, &snapshot.procs);
                    app_rows.insert(group.id.clone(), row_widget);
                }
            }

            // Remove rows for dead groups
            let live_ids: std::collections::HashSet<&AppId> =
                snapshot.groups.iter().map(|g| &g.id).collect();
            app_rows.retain(|id, row| {
                let alive = live_ids.contains(id);
                if !alive {
                    if let Some(parent) = row.container.parent() {
                        if let Some(bx) = parent.downcast_ref::<gtk::Box>() {
                            bx.remove(&row.container);
                        }
                    }
                }
                alive
            });
        }

        // Check if list order should be frozen (when cursor is hovering list)
        let is_hovered = *self.is_hovered.borrow();
        let is_first = self.last_snapshot.borrow().is_none();

        *self.last_snapshot.borrow_mut() = Some(snapshot.clone());

        if !is_hovered || is_first {
            Self::relayout_groups(
                &snapshot,
                *self.sort_by.borrow(),
                &q,
                *self.show_kernel.borrow(),
                &self.app_rows,
                &self.apps_container,
                &self.background_container,
                &self.system_container,
                &self.kernel_container,
                bg_rev,
                sys_rev,
                kern_rev,
                &self.toast_overlay,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn relayout_groups(
        snapshot: &Snapshot,
        sort: SortBy,
        search_query: &str,
        show_kernel: bool,
        app_rows: &Rc<RefCell<HashMap<AppId, AppRowWidget>>>,
        apps_container: &gtk::Box,
        background_container: &gtk::Box,
        system_container: &gtk::Box,
        kernel_container: &gtk::Box,
        bg_revealed: bool,
        sys_revealed: bool,
        kern_revealed: bool,
        toast_overlay: &adw::ToastOverlay,
    ) {
        let q = search_query.trim().to_lowercase();
        let mut groups: Vec<&AppGroup> = snapshot.groups.iter().collect();

        // Sort groups
        groups.sort_by(|a, b| match sort {
            SortBy::Cpu => b.cpu_permille.cmp(&a.cpu_permille).then(b.mem_kib.cmp(&a.mem_kib)),
            SortBy::Memory => b.mem_kib.cmp(&a.mem_kib),
            SortBy::Count => b.members.len().cmp(&a.members.len()),
            SortBy::Name => a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()),
        });

        let mut app_rows = app_rows.borrow_mut();

        let toast_fn: ToastCallback = {
            let overlay = toast_overlay.clone();
            Rc::new(move |msg: &str| {
                overlay.add_toast(adw::Toast::new(msg));
            })
        };

        let mut prev_apps: Option<gtk::Widget> = None;
        let mut prev_bg: Option<gtk::Widget> = None;
        let mut prev_sys: Option<gtk::Widget> = None;
        let mut prev_kern: Option<gtk::Widget> = None;

        for group in groups {
            if group.section == Section::Kernel && !show_kernel {
                continue;
            }

            let is_sec_visible = match group.section {
                Section::Apps => true,
                Section::Background => bg_revealed || !q.is_empty(),
                Section::System => sys_revealed || !q.is_empty(),
                Section::Kernel => kern_revealed || !q.is_empty(),
            };

            if !is_sec_visible {
                continue;
            }

            // Search filter
            let mut matches_search = q.is_empty() || group.name.to_lowercase().contains(&q);
            if !matches_search && !q.is_empty() {
                // Check if any child process matches
                for key in &group.members {
                    let pid_str = key.pid.to_string();
                    if pid_str.contains(&q) {
                        matches_search = true;
                        break;
                    }
                    if let Some(entry) = snapshot.procs.get(&key.pid) {
                        if entry.info.comm.to_lowercase().contains(&q)
                            || entry.info.display_cmdline().to_lowercase().contains(&q)
                        {
                            matches_search = true;
                            break;
                        }
                    }
                }
            }

            let row = app_rows.entry(group.id.clone()).or_insert_with(|| {
                let mut w = AppRowWidget::new(group.clone(), toast_fn.clone());
                w.update(group, &snapshot.procs);
                w
            });

            row.container.set_visible(matches_search);

            if !matches_search {
                continue;
            }

            // Auto-expand if search query matched a child process
            if !q.is_empty() {
                row.set_expanded(true);
                row.sync_children(&snapshot.procs);
            }

            let (target_container, prev_sibling_ref) = match group.section {
                Section::Apps => (apps_container, &mut prev_apps),
                Section::Background => (background_container, &mut prev_bg),
                Section::System => (system_container, &mut prev_sys),
                Section::Kernel => (kernel_container, &mut prev_kern),
            };

            // Attach to target container in order with minimal moves
            if row.container.parent().as_ref() != Some(target_container.upcast_ref()) {
                if let Some(old_parent) = row.container.parent() {
                    if let Some(bx) = old_parent.downcast_ref::<gtk::Box>() {
                        bx.remove(&row.container);
                    }
                }
                match prev_sibling_ref.as_ref() {
                    Some(prev) => target_container.insert_child_after(&row.container, Some(prev)),
                    None => target_container.prepend(&row.container),
                }
            } else {
                let current_prev = row.container.prev_sibling();
                if current_prev.as_ref() != prev_sibling_ref.as_ref() {
                    target_container.reorder_child_after(&row.container, prev_sibling_ref.as_ref());
                }
            }

            *prev_sibling_ref = Some(row.container.clone().upcast());
        }
    }
}
