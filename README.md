<p align="center">
  <img src="crates/linvclip-ui/src-tauri/icons/icon.png" width="96" alt="LinVClipBoard icon" />
</p>

<h1 align="center">LinVClipBoard</h1>

<p align="center">
  <strong>A Win+V style clipboard manager for Linux.</strong><br/>
  Fast, lightweight, works on X11 and Wayland.<br/>
  <em>v2.0 — i18n, emoji &amp; symbol pickers, zoom, redesigned settings</em>
</p>

<p align="center">
  <a href="#installation">Install</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#configuration">Config</a> &middot;
  <a href="https://github.com/akash-singh8/LinVClipBoard/releases">Releases</a>
</p>

---

## What it does

LinVClipBoard runs a background daemon (`clipd`) that captures every text and image you copy. You can search, pin, and paste from history using:

- **Overlay UI** &mdash; Press `Super+.` to open. Glassmorphism dark/light theme, keyboard-driven.
- **CLI** &mdash; `clipctl list`, `clipctl search "query"`, `clipctl paste <id>`
- **Emoji Picker** &mdash; Built-in searchable emoji grid (~300 emojis, 9 categories)
- **Symbol Picker** &mdash; Math, arrows, currency, Greek, and more
- **i18n** &mdash; English and Portuguese, easily extensible
- **Zoom** &mdash; Scale the UI from 50% to 200% (slider or Ctrl+/−/0)
- **Window Positioning** &mdash; "Fixed" (center) or "Mouse" (follows cursor)

Items are stored in a local SQLite database with FTS5 full-text search. Images are saved as PNG blobs. Everything stays under 50 MB RAM.

## Architecture

```
  linvclip-ui (Tauri)    clipctl (CLI)
        \                   /
         \                 /
      IPC (Unix Domain Socket)
              |
         clipd (daemon)
              |
     SQLite + FTS5 + blob store
              |
     arboard (X11 / Wayland)
```

| Component       | Description                              |
|-----------------|------------------------------------------|
| `clipd`         | Background daemon. Captures clipboard, enforces limits. |
| `clipctl`       | CLI client. List, search, paste, pin, delete. |
| `linvclip-ui`   | Tauri overlay window. Toggled with `Super+.` |
| `shared`        | Library crate: database, IPC, config, models. |

## Installation

### From .deb package (recommended)

Download `linvclipboard_1.0.1-1_amd64.deb` from [Releases](https://github.com/akash-singh8/LinVClipBoard/releases):

```bash
sudo dpkg -i linvclipboard_1.0.1-1_amd64.deb

# Enable the clipboard daemon
systemctl --user daemon-reload
systemctl --user enable --now clipd.service

# Verify
clipctl status
```

This single package includes everything: `clipd`, `clipctl`, `linvclip-ui`, the systemd service, desktop entry, and icon.

### Build from source

#### Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System libraries (Ubuntu/Debian)
sudo apt install -y build-essential pkg-config libsqlite3-dev \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libwayland-dev wl-clipboard

# For the Tauri UI (optional, requires Node.js >= 18)
sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

#### Build and install

```bash
# Build everything (daemon + CLI + UI) and package as .deb
make deb
sudo dpkg -i target/debian/linvclipboard_1.0.1-1_amd64.deb

# Or install directly without packaging
make build-all
make install
```

## Usage

### Overlay UI

Press **Super+.** (Win+Period) to toggle the overlay. Use arrow keys to navigate, Enter to paste, Escape to dismiss.

The overlay has three tabs:
- **Clipboard** &mdash; History with type filters (All / Text / Images / Files / Pinned)
- **Emojis** &mdash; Searchable emoji grid with categories and recently used
- **Symbols** &mdash; Math, arrows, currency, Greek letters, etc.

Keyboard shortcuts:
- `Ctrl++` / `Ctrl+-` / `Ctrl+0` &mdash; Zoom in / out / reset

### CLI

```bash
clipctl list                  # Recent items
clipctl list --limit 50       # Last 50 items
clipctl search "hello"        # Full-text search
clipctl paste <id>            # Copy item back to clipboard
clipctl pin <id>              # Pin / unpin an item
clipctl delete <id>           # Delete an item
clipctl clear                 # Clear all non-pinned items
clipctl status                # Daemon uptime, item count, DB size
```

### Daemon management

```bash
systemctl --user status clipd         # Check status
systemctl --user restart clipd        # Restart
journalctl --user -u clipd -f         # Live logs
```

## Configuration

Config file: `~/.config/linvclip/config.toml` (auto-created on first run)

```toml
[daemon]
poll_interval_ms = 250
log_level = "info"              # trace | debug | info | warn | error

[storage]
max_items = 10000
max_item_size_bytes = 52428800  # 50 MB
expiry_days = 30

[security]
blacklisted_apps = ["keepassxc", "1password", "bitwarden"]
incognito = false

[ui]
theme = "auto"                  # auto | dark | light
window_width = 420
window_height = 520
language = "en"                 # en | pt (add your own!)
zoom = 100                      # 50–200
window_position = "mouse"       # mouse | fixed
```

## Uninstall

```bash
systemctl --user stop clipd.service
systemctl --user disable clipd.service
sudo dpkg -r linvclipboard

# Remove user data (optional)
rm -rf ~/.config/linvclip ~/.local/share/linvclip
```

## Adding a New Language

1. Copy `crates/linvclip-ui/src/i18n/en.json` to `<code>.json` (e.g. `fr.json`)
2. Translate all values (keep the keys unchanged)
3. Import the file in `crates/linvclip-ui/src/i18n/index.jsx` and add it to the `TRANSLATIONS` map
4. The new language will appear automatically in Settings

## License

[MIT](LICENSE)
