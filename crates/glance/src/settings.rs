//! Persistent application settings for Glance.

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub autostart: bool,
    pub confirm_quit_app: bool,
    pub show_kernel: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: true,
            confirm_quit_app: true,
            show_kernel: false,
        }
    }
}

impl Settings {
    fn config_path() -> Option<PathBuf> {
        let config_dir = dirs_config_dir()?;
        Some(config_dir.join("glance").join("settings.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        let default = Self::default();
        default.save();
        default
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Synchronize the autostart .desktop file in ~/.config/autostart.
    pub fn sync_autostart(&self) {
        let Some(config_dir) = dirs_config_dir() else {
            return;
        };
        let autostart_dir = config_dir.join("autostart");
        let autostart_file = autostart_dir.join("io.github.maycon.Glance.desktop");

        if self.autostart {
            let _ = fs::create_dir_all(&autostart_dir);
            let exec_path = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "glance".to_string());
            let desktop_content = format!(
                "[Desktop Entry]\n\
Type=Application\n\
Name=Glance\n\
Comment=Lightweight real-time Linux process and application monitor\n\
Exec={} --hidden\n\
Icon=io.github.maycon.Glance\n\
Terminal=false\n\
Categories=System;Monitor;Utility;\n\
StartupNotify=false\n\
X-GNOME-Autostart-enabled=true\n",
                exec_path
            );
            let _ = fs::write(autostart_file, desktop_content);
        } else if autostart_file.exists() {
            let _ = fs::remove_file(autostart_file);
        }
    }
}

fn dirs_config_dir() -> Option<PathBuf> {
    if let Ok(cfg) = std::env::var("XDG_CONFIG_HOME") {
        if !cfg.is_empty() {
            return Some(PathBuf::from(cfg));
        }
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config"))
}
