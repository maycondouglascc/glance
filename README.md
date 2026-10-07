# Glance

A lightweight, high-performance Linux desktop application for Ubuntu and Linux desktops that provides a real-time, grouped view of running applications and their underlying processes.

## Features

- **Application Grouping**: Groups related Linux processes under their parent application using systemd cgroup scopes (`.scope`), Flatpak/Snap identifiers, and desktop entry (`.desktop`) index mappings.
- **Progressive Disclosure**: Notion-style toggle rows displaying app-level CPU, RAM, and process counts by default; expands smoothly to reveal child processes (Renderer, Extensions, Zygote, Helpers, Scripts).
- **Process Inspection**: Detailed popover showing command line, executable path, start time, UID, thread count, and PSS/RSS memory breakdown.
- **Safe Termination**:
  - Distinguishes graceful (`SIGTERM`) and forced (`SIGKILL`) termination.
  - Race-free `pidfd_send_signal` with process start-time verification to prevent PID-reuse accidents.
  - Systemd `KillUnit` support for atomic group termination.
  - Safety tiers: safeguards critical session components (`gnome-shell`, `wireplumber`, etc.) and blocks system-critical processes (PID 1, kernel threads).
- **Performance**:
  - Zero-allocation hot path reading `/proc` directly (never shells out to `ps` or `top`).
  - Open file descriptor cache (`pread`) per active process.
  - 869 KB release binary.
  - Sub-5ms scan time for 500+ processes.
  - Adaptive refresh cadence (1s focused, 2s unfocused, paused when hidden).
- **Search & Sort**: Filter by name, command line, or PID; sort by CPU, RAM, process count, or name.

## Workspace Layout

```
glance/
├── Cargo.toml
├── Makefile
├── data/
│   ├── io.github.maycon.Glance.desktop
│   └── io.github.maycon.Glance.metainfo.xml
└── crates/
    ├── glance-proc/      # /proc scanner, tiered reads, pidfd signals (no GTK)
    ├── glance-group/     # cgroup leaf parser, desktop index, resolver chain (no GTK)
    ├── glance-cli/       # glance-tree terminal tree viewer & benchmark spike
    └── glance/           # GTK4 + libadwaita desktop utility
```

## Installation

### 1. Universal One-Line Installer (Recommended)

Quickly install Glance to `~/.local/bin` (no root required):

```bash
curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh
```

Or install system-wide to `/usr/local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sudo sh
```

### 2. Ubuntu / Debian (.deb)

Download and install the latest `.deb` package directly from [GitHub Releases](https://github.com/maycondouglascc/glance/releases/latest):

```bash
curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance_0.1.0_amd64.deb
sudo apt install ./glance_0.1.0_amd64.deb
```

### 3. Arch Linux / Manjaro

Clone and build using the provided PKGBUILD:

```bash
git clone https://github.com/maycondouglascc/glance.git
cd glance/packaging/aur
makepkg -si
```

### 4. Tarball (Any Linux Distribution)

Download and extract pre-compiled binaries:

```bash
curl -LO https://github.com/maycondouglascc/glance/releases/latest/download/glance-linux-x86_64.tar.gz
tar -xzf glance-linux-x86_64.tar.gz
install -m 755 glance ~/.local/bin/glance
install -m 755 glance-tree ~/.local/bin/glance-tree
```

### 5. Build from Source

```bash
git clone https://github.com/maycondouglascc/glance.git
cd glance
make install
```

Or run via Cargo:
```bash
cargo run --release -p glance
```

### CLI Tree Viewer

```bash
cargo run --release -p glance-cli -- --help
glance-tree -c           # Print expanded process tree
glance-tree --sort mem   # Sort by memory usage
glance-tree --bench 200  # Benchmark scanner and grouper over 200 iterations
```

## Keyboard Shortcuts

- **Type anywhere**: Instant search filter
- **Space / Enter**: Open process details dialog
- **Double click row**: Expand / collapse application processes
- **Delete**: End process / Quit application (SIGTERM)
- **Right Click**: Context menu (Inspect, Terminate, Force Kill, Copy PID / Cmdline)
