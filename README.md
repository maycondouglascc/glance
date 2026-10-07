# Glance

[![Release](https://img.shields.io/github/v/release/maycondouglascc/glance?color=blue&label=release)](https://github.com/maycondouglascc/glance/releases)
[![Build Status](https://github.com/maycondouglascc/glance/actions/workflows/release.yml/badge.svg)](https://github.com/maycondouglascc/glance/actions/workflows/release.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![GTK4](https://img.shields.io/badge/GTK-4.14%2B-green.svg)](https://gtk.org)
[![Libadwaita](https://img.shields.io/badge/libadwaita-1.5%2B-purple.svg)](https://gnome.pages.gitlab.gnome.org/libadwaita/)

A lightweight, high-performance Linux desktop application and process monitor designed for Linux desktops (GNOME, COSMIC, KDE Plasma, etc.). Glance provides a real-time, grouped view of running applications and their underlying processes using systemd cgroup scopes and progressive disclosure.

---

## Highlights & Performance

- **Ultra-Lightweight**: Only **~65 MB RSS** in background tray mode, and **~90 MB RSS** when active.
- **Zero Idle CPU**: Drops to **0.00% CPU** when minimized or in the system tray via kernel-blocking event waits.
- **Sub-5ms Scans**: Direct, zero-allocation `/proc` reads for 500+ processes with open file descriptor caching. Never shells out to `ps` or `top`.
- **Hardware-Friendly**: Cairo 2D accelerated rendering by default eliminates GPU/Vulkan driver overhead.
- **Modern Memory Management**: Backed by `mimalloc` with aggressive heap trimming on window hide.

---

## Features

- **Application Grouping**: Groups related Linux processes under their parent application using systemd cgroup scopes (`.scope`), Flatpak/Snap identifiers, and desktop entry (`.desktop`) index mappings.
- **Progressive Disclosure**: Notion-style toggle rows displaying app-level CPU, RAM, and process counts; expands smoothly to reveal child processes (Renderer, Extensions, Zygote, Helpers, Scripts).
- **Process Inspection**: Detailed view showing command line, executable path, start time, UID, thread count, and PSS/RSS memory breakdown.
- **Safe Termination**:
  - Distinguishes graceful (`SIGTERM`) and forced (`SIGKILL`) termination.
  - Race-free `pidfd_send_signal` with process start-time verification to prevent PID-reuse accidents.
  - Systemd `KillUnit` support for atomic group termination.
  - Safety tiers: safeguards critical session components (`gnome-shell`, `wireplumber`, etc.) and blocks system-critical processes (PID 1, kernel threads).
- **Search & Filter**: Type anywhere for instant live search by name, executable, PID, or command line arguments.
- **Adaptive Cadence**: 1-second cadence when focused, 2 seconds when unfocused, completely paused when minimized to tray.
- **System Tray Support**: Runs as a daemon in the background with StatusNotifierItem / AppIndicator support.

---

## Installation

### 1. Universal One-Line Installer (Recommended)

Quickly install the latest release directly to `~/.local/bin` (no root required):

```bash
curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh
```

Or install system-wide to `/usr/local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sudo sh
```

### 2. Ubuntu / Debian / Pop!_OS / Linux Mint (`.deb`)

Download and install the native Debian package directly from [GitHub Releases](https://github.com/maycondouglascc/glance/releases/latest):

```bash
# Download and install the latest .deb package
curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance_0.1.0_amd64.deb
sudo apt install ./glance_0.1.0_amd64.deb
```

*Or via GitHub CLI:*
```bash
gh release download --repo maycondouglascc/glance --pattern "*.deb"
sudo apt install ./glance_*_amd64.deb
```

### 3. Arch Linux / Manjaro (AUR / PKGBUILD)

Build and install using the provided PKGBUILD:

```bash
git clone https://github.com/maycondouglascc/glance.git
cd glance/packaging/aur
makepkg -si
```

### 4. Standalone Tarball (Fedora, openSUSE, Alpine, etc.)

Download pre-compiled binaries:

```bash
curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance-linux-x86_64.tar.gz
tar -xzf glance-linux-x86_64.tar.gz
install -m 755 glance ~/.local/bin/glance
install -m 755 glance-tree ~/.local/bin/glance-tree
```

### 5. Build from Source

Requirements: Rust 1.85+, `libgtk-4-dev`, `libadwaita-1-dev`.

```bash
git clone https://github.com/maycondouglascc/glance.git
cd glance
make install
```

Or install directly with `cargo`:

```bash
cargo install --git https://github.com/maycondouglascc/glance.git glance
```

---

## Usage

### Desktop Application

Launch Glance from your application menu or run from the terminal:

```bash
glance
```

To start Glance in background / system tray daemon mode on system startup:

```bash
glance --hidden
```

### CLI Process Tree (`glance-tree`)

Glance includes a standalone high-speed terminal tree viewer:

```bash
glance-tree               # Print grouped process tree summary
glance-tree -c            # Print tree with expanded child processes
glance-tree --sort mem    # Sort applications by memory usage
glance-tree --sort cpu    # Sort applications by CPU usage
glance-tree --sort count  # Sort applications by process count
glance-tree --pss         # Calculate Proportional Set Size (PSS) memory
glance-tree --bench 200   # Run 200 benchmark iterations of /proc scanner
```

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| **Type anywhere** | Instant search filter |
| **Space** / **Enter** | Open process details & telemetry modal |
| **Double Click** | Expand / collapse application process tree |
| **Delete** | Gracefully terminate selected application or process (`SIGTERM`) |
| **Shift + Delete** | Force kill selected application or process (`SIGKILL`) |
| **Right Click** | Context menu (Inspect, Terminate, Force Kill, Copy PID / Cmdline) |
| **Escape** | Clear search filter or close modal |

---

## Uninstallation

To remove Glance installed via the one-line installer:

```bash
curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh -s -- --uninstall
```

Or if installed from source:

```bash
make uninstall
```

Or if installed via Debian package:

```bash
sudo apt remove glance
```

---

## Workspace Layout

```
glance/
├── Cargo.toml
├── Makefile
├── install.sh                          # Universal one-line installer
├── data/
│   ├── io.github.maycon.Glance.desktop # Desktop entry
│   └── io.github.maycon.Glance.metainfo.xml
├── packaging/
│   └── aur/PKGBUILD                    # Arch Linux package build
└── crates/
    ├── glance-proc/                    # /proc scanner, tiered reads, pidfd signals (no GTK)
    ├── glance-group/                   # cgroup leaf parser, desktop index, resolver chain (no GTK)
    ├── glance-cli/                     # glance-tree terminal tree viewer & benchmark spike
    └── glance/                         # GTK4 + libadwaita desktop utility
```

---

## License

Glance is open-source software licensed under the [GNU General Public License v3.0](LICENSE).
