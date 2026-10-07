#![allow(dead_code)]
use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use glance_group::format;
use glance_group::{AppGroup, Protection};
use glance_proc::signal::Sig;
use glance_proc::{ProcEntry, ProcKey};

use crate::actions::{kill_group, kill_proc, outcome_message};
use crate::details::show_process_details;
use crate::dialogs::{confirm_app_termination, confirm_termination};

pub type ToastCallback = Rc<dyn Fn(&str)>;

/// Widget for an individual process under an application.
pub struct ProcRowWidget {
    pub container: gtk::Box,
    pub key: ProcKey,
    cpu_label: gtk::Label,
    mem_label: gtk::Label,
    name_label: gtk::Label,
    last_cpu: Option<u16>,
    last_rss: u64,
    last_label: String,
}

impl ProcRowWidget {
    pub fn new(
        entry: &ProcEntry,
        label: &str,
        protection: Protection,
        on_toast: ToastCallback,
    ) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        container.set_margin_top(2);
        container.set_margin_bottom(2);
        container.set_margin_start(0);
        container.set_margin_end(0);
        container.add_css_class("proc-row");

        let title_str = format!("{label}  ·  {}", entry.key.pid);
        let name_label = gtk::Label::builder()
            .label(&title_str)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .halign(gtk::Align::Start)
            .build();
        name_label.add_css_class("proc-name");

        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let cpu_str = entry
            .cpu_permille
            .map_or("—".to_string(), |c| format::cpu(u32::from(c)));
        let cpu_label = gtk::Label::builder()
            .label(&cpu_str)
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        cpu_label.set_width_request(36);
        cpu_label.add_css_class("numeric");
        cpu_label.add_css_class("proc-metric");

        let mem_str = format::mem(entry.sample.rss_kib);
        let mem_label = gtk::Label::builder()
            .label(&mem_str)
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        mem_label.set_width_request(38);
        mem_label.set_margin_start(24);
        mem_label.add_css_class("numeric");
        mem_label.add_css_class("proc-metric");

        // Info / Details button
        let details_btn = gtk::Button::new();
        details_btn.set_child(Some(&crate::icons::info_icon()));
        details_btn.add_css_class("action-pill-btn");
        details_btn.add_css_class("circular");
        details_btn.set_valign(gtk::Align::Center);
        details_btn.set_tooltip_text(Some("Inspect process details"));
        let entry_clone = entry.clone();
        let label_owned = label.to_string();
        details_btn.connect_clicked(move |btn| {
            show_process_details(btn, &entry_clone, &label_owned);
        });

        // End process button (SIGTERM)
        let end_btn = gtk::Button::from_icon_name("media-playback-stop-symbolic");
        end_btn.add_css_class("action-pill-btn");
        end_btn.add_css_class("circular");
        end_btn.set_valign(gtk::Align::Center);
        end_btn.set_tooltip_text(Some("End process (SIGTERM)"));

        let key = entry.key;
        let proc_name = label.to_string();
        let on_toast_clone = on_toast.clone();
        end_btn.connect_clicked(move |btn| {
            let toast_fn = on_toast_clone.clone();
            let name_for_toast = proc_name.clone();
            confirm_termination(btn, &proc_name, protection, Sig::Term, move || {
                let outcome = kill_proc(key, Sig::Term);
                toast_fn(&outcome_message(&name_for_toast, Sig::Term, outcome));
            });
        });

        // Force kill in context menu
        let menu_model = gio::Menu::new();
        menu_model.append(Some("Inspect Details"), Some("proc.details"));
        menu_model.append(Some("End Process (SIGTERM)"), Some("proc.term"));
        menu_model.append(Some("Force Kill (SIGKILL)"), Some("proc.kill"));
        menu_model.append(Some("Copy PID"), Some("proc.copy_pid"));
        menu_model.append(Some("Copy Command Line"), Some("proc.copy_cmd"));

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        popover_menu.set_parent(&container);

        // Right-click gesture
        let right_click = gtk::GestureClick::new();
        right_click.set_button(gdk4::BUTTON_SECONDARY);
        let popover_clone = popover_menu.clone();
        right_click.connect_pressed(move |_, _, x, y| {
            let rect = gdk4::Rectangle::new(x as i32, y as i32, 1, 1);
            popover_clone.set_pointing_to(Some(&rect));
            popover_clone.popup();
        });
        container.add_controller(right_click);

        // Action group for menu
        let action_group = gio::SimpleActionGroup::new();
        let entry_for_details = entry.clone();
        let label_for_details = label.to_string();
        let action_details = gio::SimpleAction::new("details", None);
        let container_weak = container.downgrade();
        action_details.connect_activate(move |_, _| {
            if let Some(c) = container_weak.upgrade() {
                show_process_details(&c, &entry_for_details, &label_for_details);
            }
        });
        action_group.add_action(&action_details);

        let action_term = gio::SimpleAction::new("term", None);
        let on_toast_term = on_toast.clone();
        let name_term = label.to_string();
        let container_weak2 = container.downgrade();
        action_term.connect_activate(move |_, _| {
            if let Some(c) = container_weak2.upgrade() {
                let toast_fn = on_toast_term.clone();
                let name_for_toast = name_term.clone();
                confirm_termination(&c, &name_term, protection, Sig::Term, move || {
                    let out = kill_proc(key, Sig::Term);
                    toast_fn(&outcome_message(&name_for_toast, Sig::Term, out));
                });
            }
        });
        action_group.add_action(&action_term);

        let action_kill = gio::SimpleAction::new("kill", None);
        let on_toast_kill = on_toast.clone();
        let name_kill = label.to_string();
        let container_weak3 = container.downgrade();
        action_kill.connect_activate(move |_, _| {
            if let Some(c) = container_weak3.upgrade() {
                let toast_fn = on_toast_kill.clone();
                let name_for_toast = name_kill.clone();
                confirm_termination(&c, &name_kill, protection, Sig::Kill, move || {
                    let out = kill_proc(key, Sig::Kill);
                    toast_fn(&outcome_message(&name_for_toast, Sig::Kill, out));
                });
            }
        });
        action_group.add_action(&action_kill);

        let pid_num = entry.key.pid;
        let action_copy_pid = gio::SimpleAction::new("copy_pid", None);
        let container_weak4 = container.downgrade();
        action_copy_pid.connect_activate(move |_, _| {
            if let Some(c) = container_weak4.upgrade() {
                c.display().clipboard().set_text(&format!("{pid_num}"));
            }
        });
        action_group.add_action(&action_copy_pid);

        let cmdline = entry.info.display_cmdline();
        let action_copy_cmd = gio::SimpleAction::new("copy_cmd", None);
        let container_weak5 = container.downgrade();
        action_copy_cmd.connect_activate(move |_, _| {
            if let Some(c) = container_weak5.upgrade() {
                c.display().clipboard().set_text(&cmdline);
            }
        });
        action_group.add_action(&action_copy_cmd);

        container.insert_action_group("proc", Some(&action_group));

        let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        actions_box.set_width_request(44);
        actions_box.set_margin_start(16);
        actions_box.set_halign(gtk::Align::End);
        actions_box.append(&details_btn);
        actions_box.append(&end_btn);

        let metrics_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        metrics_box.append(&cpu_label);
        metrics_box.append(&mem_label);
        metrics_box.append(&actions_box);

        container.append(&name_label);
        container.append(&spacer);
        container.append(&metrics_box);

        Self {
            container,
            key: entry.key,
            cpu_label,
            mem_label,
            name_label,
            last_cpu: entry.cpu_permille,
            last_rss: entry.sample.rss_kib,
            last_label: label.to_string(),
        }
    }

    pub fn update(&mut self, entry: &ProcEntry, label: &str) {
        if self.last_cpu != entry.cpu_permille {
            self.last_cpu = entry.cpu_permille;
            let cpu_str = entry
                .cpu_permille
                .map_or("—".to_string(), |c| format::cpu(u32::from(c)));
            self.cpu_label.set_text(&cpu_str);
        }

        if self.last_rss != entry.sample.rss_kib {
            self.last_rss = entry.sample.rss_kib;
            let mem_str = format::mem(entry.sample.rss_kib);
            self.mem_label.set_text(&mem_str);
        }

        if self.last_label != label {
            self.last_label.clear();
            self.last_label.push_str(label);
            let title_str = format!("{label}  ·  {}", entry.key.pid);
            self.name_label.set_text(&title_str);
        }
    }
}

/// Notion-style progressive disclosure widget for an Application Group.
pub struct AppRowWidget {
    pub container: gtk::Box,
    pub header_box: gtk::Box,
    pub children_box: gtk::Box,
    pub revealer: gtk::Revealer,
    pub expand_btn: gtk::Button,
    pub is_expanded: Rc<RefCell<bool>>,
    cpu_label: gtk::Label,
    mem_label: gtk::Label,
    count_label: gtk::Label,
    title_label: gtk::Label,
    pub proc_widgets: Vec<ProcRowWidget>,
    pub group: AppGroup,
    on_toast: ToastCallback,
    last_cpu: u32,
    last_mem: u64,
    last_count: usize,
    last_is_pss: bool,
}

impl AppRowWidget {
    pub fn new(group: AppGroup, on_toast: ToastCallback) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.add_css_class("app-group-card");

        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header_box.set_margin_top(2);
        header_box.set_margin_bottom(2);
        header_box.set_margin_start(0);
        header_box.set_margin_end(0);
        header_box.add_css_class("app-header-row");

        let is_expanded = Rc::new(RefCell::new(false));

        // Disclosure button (▸ / ▾)
        let expand_btn = gtk::Button::from_icon_name("pan-end-symbolic");
        expand_btn.add_css_class("chevron-btn");
        expand_btn.add_css_class("circular");
        expand_btn.set_valign(gtk::Align::Center);
        expand_btn.set_tooltip_text(Some("Expand processes"));

        // Icon
        let icon_name = group.icon.as_deref().unwrap_or("application-x-executable");
        let icon_img = gtk::Image::from_icon_name(icon_name);
        icon_img.set_pixel_size(20);
        icon_img.add_css_class("app-icon");
        icon_img.set_valign(gtk::Align::Center);

        // Title
        let title_label = gtk::Label::builder()
            .label(&group.name)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .halign(gtk::Align::Start)
            .build();
        title_label.add_css_class("app-title");

        // Title & icon group (gap 4px in Figma)
        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        title_box.append(&icon_img);
        title_box.append(&title_label);

        // Process count pill
        let count_str = format!("{} procs", group.members.len());
        let count_label = gtk::Label::builder()
            .label(&count_str)
            .halign(gtk::Align::Start)
            .build();
        count_label.add_css_class("proc-count");

        // Info group (title_box, optional badge, count_label with gap 8px in Figma)
        let info_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        info_box.append(&title_box);
        if let Some(badge) = group.badge {
            let badge_lbl = gtk::Label::new(Some(badge));
            badge_lbl.add_css_class("badge");
            info_box.append(&badge_lbl);
        }
        info_box.append(&count_label);

        // Left section (chevron + info_box with gap 4px in Figma)
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        left_box.append(&expand_btn);
        left_box.append(&info_box);

        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        // CPU
        let cpu_str = format::cpu(group.cpu_permille);
        let cpu_label = gtk::Label::builder()
            .label(&cpu_str)
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        cpu_label.set_width_request(36);
        cpu_label.add_css_class("numeric");
        cpu_label.add_css_class("app-metric");

        // Memory
        let approx = if group.mem_is_pss { "" } else { "≈" };
        let mem_str = format!("{approx}{}", format::mem(group.mem_kib));
        let mem_label = gtk::Label::builder()
            .label(&mem_str)
            .halign(gtk::Align::End)
            .xalign(1.0)
            .build();
        mem_label.set_width_request(38);
        mem_label.set_margin_start(24);
        mem_label.add_css_class("numeric");
        mem_label.add_css_class("app-metric");

        // Quick End Application button
        let end_btn = gtk::Button::from_icon_name("window-close-symbolic");
        end_btn.add_css_class("action-pill-btn");
        end_btn.add_css_class("circular");
        end_btn.set_valign(gtk::Align::Center);
        end_btn.set_tooltip_text(Some("Quit Application (SIGTERM)"));

        let group_clone = group.clone();
        let on_toast_clone = on_toast.clone();
        end_btn.connect_clicked(move |btn| {
            let toast_fn = on_toast_clone.clone();
            let grp = group_clone.clone();
            confirm_app_termination(btn, &group_clone.name, group_clone.protection, Sig::Term, move || {
                let outcome = kill_group(&grp, Sig::Term);
                toast_fn(&outcome_message(&grp.name, Sig::Term, outcome));
            });
        });

        let actions_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        actions_box.set_width_request(44);
        actions_box.set_margin_start(16);
        actions_box.set_halign(gtk::Align::End);

        let end_btn_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        end_btn_spacer.set_width_request(24);
        actions_box.append(&end_btn_spacer);
        actions_box.append(&end_btn);

        let metrics_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        metrics_box.append(&cpu_label);
        metrics_box.append(&mem_label);
        metrics_box.append(&actions_box);

        header_box.append(&left_box);
        header_box.append(&spacer);
        header_box.append(&metrics_box);

        // Children revealer container
        let revealer = gtk::Revealer::new();
        revealer.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(0);

        let children_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        children_box.add_css_class("proc-children-container");
        revealer.set_child(Some(&children_box));

        // Connect expand button toggle
        let toggle_expand = {
            let revealer = revealer.clone();
            let expand_btn = expand_btn.clone();
            let is_expanded = is_expanded.clone();
            Rc::new(move || {
                let cur = *is_expanded.borrow();
                *is_expanded.borrow_mut() = !cur;
                revealer.set_reveal_child(!cur);
                if !cur {
                    expand_btn.set_icon_name("pan-down-symbolic");
                } else {
                    expand_btn.set_icon_name("pan-end-symbolic");
                }
            })
        };

        let toggle_expand_btn = toggle_expand.clone();
        expand_btn.connect_clicked(move |_| {
            toggle_expand_btn();
        });

        // Single click on row body also toggles
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(gdk4::BUTTON_PRIMARY);
        let toggle_expand_body = toggle_expand.clone();
        let end_btn_weak = end_btn.downgrade();
        let expand_btn_weak = expand_btn.downgrade();
        let header_box_weak = header_box.downgrade();
        click_gesture.connect_pressed(move |_, n_press, x, y| {
            if n_press == 1 {
                if let (Some(header), Some(end_b), Some(exp_b)) =
                    (header_box_weak.upgrade(), end_btn_weak.upgrade(), expand_btn_weak.upgrade())
                {
                    if let Some(picked) = header.pick(x, y, gtk::PickFlags::DEFAULT) {
                        // Ignore clicks directly on or inside action buttons
                        if picked == end_b || picked.is_ancestor(&end_b) {
                            return;
                        }
                        if picked == exp_b || picked.is_ancestor(&exp_b) {
                            return;
                        }
                    }
                    toggle_expand_body();
                }
            }
        });
        header_box.set_cursor_from_name(Some("pointer"));
        header_box.add_controller(click_gesture);

        // Context menu for application group
        let menu_model = gio::Menu::new();
        menu_model.append(Some("Quit Application (SIGTERM)"), Some("app.term"));
        menu_model.append(Some("Force Quit Application (SIGKILL)"), Some("app.kill"));
        menu_model.append(Some("Copy Application Name"), Some("app.copy_name"));

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu_model));
        popover_menu.set_parent(&header_box);

        let right_click = gtk::GestureClick::new();
        right_click.set_button(gdk4::BUTTON_SECONDARY);
        let popover_clone = popover_menu.clone();
        right_click.connect_pressed(move |_, _, x, y| {
            let rect = gdk4::Rectangle::new(x as i32, y as i32, 1, 1);
            popover_clone.set_pointing_to(Some(&rect));
            popover_clone.popup();
        });
        header_box.add_controller(right_click);

        let action_group = gio::SimpleActionGroup::new();
        let group_term = group.clone();
        let on_toast_term = on_toast.clone();
        let header_weak = header_box.downgrade();
        let action_term = gio::SimpleAction::new("term", None);
        action_term.connect_activate(move |_, _| {
            if let Some(h) = header_weak.upgrade() {
                let toast_fn = on_toast_term.clone();
                let grp = group_term.clone();
                confirm_app_termination(&h, &group_term.name, group_term.protection, Sig::Term, move || {
                    let out = kill_group(&grp, Sig::Term);
                    toast_fn(&outcome_message(&grp.name, Sig::Term, out));
                });
            }
        });
        action_group.add_action(&action_term);

        let group_kill = group.clone();
        let on_toast_kill = on_toast.clone();
        let header_weak2 = header_box.downgrade();
        let action_kill = gio::SimpleAction::new("kill", None);
        action_kill.connect_activate(move |_, _| {
            if let Some(h) = header_weak2.upgrade() {
                let toast_fn = on_toast_kill.clone();
                let grp = group_kill.clone();
                confirm_app_termination(&h, &group_kill.name, group_kill.protection, Sig::Kill, move || {
                    let out = kill_group(&grp, Sig::Kill);
                    toast_fn(&outcome_message(&grp.name, Sig::Kill, out));
                });
            }
        });
        action_group.add_action(&action_kill);

        let name_to_copy = group.name.clone();
        let header_weak3 = header_box.downgrade();
        let action_copy_name = gio::SimpleAction::new("copy_name", None);
        action_copy_name.connect_activate(move |_, _| {
            if let Some(h) = header_weak3.upgrade() {
                h.display().clipboard().set_text(&name_to_copy);
            }
        });
        action_group.add_action(&action_copy_name);

        header_box.insert_action_group("app", Some(&action_group));

        container.append(&header_box);
        container.append(&revealer);

        Self {
            container,
            header_box,
            children_box,
            revealer,
            expand_btn,
            is_expanded,
            cpu_label,
            mem_label,
            count_label,
            title_label,
            proc_widgets: Vec::new(),
            group: group.clone(),
            on_toast,
            last_cpu: group.cpu_permille,
            last_mem: group.mem_kib,
            last_count: group.members.len(),
            last_is_pss: group.mem_is_pss,
        }
    }

    pub fn update(
        &mut self,
        group: &AppGroup,
        procs: &std::collections::HashMap<i32, ProcEntry>,
    ) {
        self.group = group.clone();

        if self.last_cpu != group.cpu_permille {
            self.last_cpu = group.cpu_permille;
            self.cpu_label.set_text(&format::cpu(group.cpu_permille));
        }

        if self.last_mem != group.mem_kib || self.last_is_pss != group.mem_is_pss {
            self.last_mem = group.mem_kib;
            self.last_is_pss = group.mem_is_pss;
            let approx = if group.mem_is_pss { "" } else { "≈" };
            self.mem_label.set_text(&format!("{approx}{}", format::mem(group.mem_kib)));
        }

        if self.last_count != group.members.len() {
            self.last_count = group.members.len();
            self.count_label.set_text(&format!("{} procs", group.members.len()));
        }

        // Only populate child widgets if expanded
        if *self.is_expanded.borrow() {
            self.sync_children(procs);
        }
    }

    pub fn set_expanded(&self, expanded: bool) {
        *self.is_expanded.borrow_mut() = expanded;
        self.revealer.set_reveal_child(expanded);
        if expanded {
            self.expand_btn.set_icon_name("pan-down-symbolic");
        } else {
            self.expand_btn.set_icon_name("pan-end-symbolic");
        }
    }

    pub fn sync_children(&mut self, procs: &std::collections::HashMap<i32, ProcEntry>) {
        let mut live_members: Vec<&ProcEntry> = self
            .group
            .members
            .iter()
            .filter_map(|k| procs.get(&k.pid))
            .collect();
        live_members.sort_by(|a, b| {
            b.cpu_permille
                .cmp(&a.cpu_permille)
                .then(b.sample.rss_kib.cmp(&a.sample.rss_kib))
        });

        // Retain or update existing widgets
        let mut new_widgets = Vec::with_capacity(live_members.len());
        let mut prev_child: Option<gtk::Widget> = None;

        for entry in live_members {
            let label = glance_group::labels::process_label(&entry.info);
            let prot = glance_group::protection::classify(
                entry,
                entry.info.uid.unwrap_or(1000),
                std::process::id() as i32,
            );

            let w = if let Some(pos) = self.proc_widgets.iter().position(|w| w.key == entry.key) {
                let mut w = self.proc_widgets.swap_remove(pos);
                w.update(entry, &label);
                w
            } else {
                let w = ProcRowWidget::new(entry, &label, prot, self.on_toast.clone());
                match prev_child.as_ref() {
                    Some(prev) => self.children_box.insert_child_after(&w.container, Some(prev)),
                    None => self.children_box.prepend(&w.container),
                }
                w
            };

            let cur_prev = w.container.prev_sibling();
            if cur_prev.as_ref() != prev_child.as_ref() {
                self.children_box.reorder_child_after(&w.container, prev_child.as_ref());
            }

            prev_child = Some(w.container.clone().upcast());
            new_widgets.push(w);
        }

        // Clean up widgets for exited children
        for old_w in self.proc_widgets.drain(..) {
            self.children_box.remove(&old_w.container);
        }
        self.proc_widgets = new_widgets;
    }
}
