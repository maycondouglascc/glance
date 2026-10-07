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

## Quick Start

### Build & Run GUI

```bash
make release
~/.local/bin/glance
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
