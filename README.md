<p align="center">
  <img src="crates/linvclip-ui/src-tauri/icons/icon.png" width="120" alt="LinVClipBoard" />
</p>

<h1 align="center">LinVClipBoard</h1>

<p align="center">
  <strong>The clipboard manager Linux deserves.</strong><br/>
  Blazing fast &bull; Keyboard-first &bull; X11 + Wayland &bull; Under 50 MB RAM
</p>

<p align="center">
  <a href="https://github.com/DreamerX00/LinVClipBoard/releases/latest"><img src="https://img.shields.io/github/v/release/DreamerX00/LinVClipBoard?style=flat-square&color=6366f1&label=release" alt="Latest Release" /></a>
  <a href="https://github.com/DreamerX00/LinVClipBoard/blob/main/LICENSE"><img src="https://img.shields.io/github/license/DreamerX00/LinVClipBoard?style=flat-square&color=34d399" alt="MIT License" /></a>
  <a href="https://github.com/DreamerX00/LinVClipBoard/releases/latest"><img src="https://img.shields.io/github/downloads/DreamerX00/LinVClipBoard/total?style=flat-square&color=f59e0b&label=downloads" alt="Downloads" /></a>
  <img src="https://img.shields.io/badge/rust-2024-orange?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/tauri-v2-24C8D8?style=flat-square&logo=tauri" alt="Tauri v2" />
</p>

<p align="center">
  <a href="#-quick-install">Install</a> &bull;
  <a href="#-features">Features</a> &bull;
  <a href="#%EF%B8%8F-usage">Usage</a> &bull;
  <a href="#-configuration">Config</a> &bull;
  <a href="https://github.com/DreamerX00/LinVClipBoard/releases">Releases</a>
</p>

---

## Why LinVClipBoard?

Windows has `Win+V`. Mac has clipboard history. Linux had…nothing great. Until now.

LinVClipBoard is a **native**, **lightweight** clipboard platform that gives you everything the other OSes have — and more. GIF search, emoji picker, symbol tables, full-text search across your entire clipboard history, all wrapped in a gorgeous glassmorphism overlay activated with a single keystroke.

Built in **Rust + Tauri v2**. Runs as a systemd user service. No Electron. No bloat.

---

## ✨ Features

| Feature | Description |
|:--------|:------------|
| 📋 **Clipboard History** | Every text & image you copy, searchable with SQLite FTS5 |
| 🎞️ **GIF Search** | Browse trending GIFs, search the KLIPY library, copy URL with one click |
| 😀 **Emoji Picker** | ~300 emojis across 9 categories with recent-used tracking |
| ∑ **Symbol Table** | Math, arrows, currency, Greek, superscripts, box drawing |
| 🔍 **Full-text Search** | Instant FTS5 search across your entire history |
| 📌 **Pin & Organize** | Pin important items so they never expire |
| 🎨 **Themes** | Dark, Light, or Auto (follows OS). Glassmorphism everywhere |
| 🌐 **4 Languages** | English, Português, 日本語, हिन्दी — easily extensible |
| 🔄 **Auto Updates** | Weekly update check with desktop notification. Manual check in Settings |
| 🖱️ **Draggable Window** | Grab the title bar and move the overlay anywhere |
| 🔍 **Zoom** | Scale the entire UI from 50% to 200% |
| ⌨️ **Keyboard-first** | Arrow keys, Enter to paste, Escape to dismiss, Ctrl+/−/0 for zoom |
| 🔒 **Secure** | Incognito mode, app blacklist, auto-expiry, memory limits |
| 🖥️ **X11 + Wayland** | Native clipboard access via arboard — no hacks |

---

## 🚀 Quick Install

### One command (Debian/Ubuntu)

```bash
# Download the latest .deb from Releases and install:
sudo dpkg -i linvclipboard_1.5.0-1_amd64.deb
```

That's it. The daemon starts automatically. Press **`Super+.`** to open the overlay.

> The package includes everything: `clipd` (daemon), `clipctl` (CLI), `linvclip-ui` (overlay), systemd service + update timer, desktop entry, and icon.

### Build from source

```bash
# Prerequisites (Ubuntu/Debian)
sudo apt install -y build-essential pkg-config libsqlite3-dev \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libwayland-dev wl-clipboard \
    libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

# Clone & build
git clone https://github.com/DreamerX00/LinVClipBoard.git
cd LinVClipBoard
make deb
sudo dpkg -i target/debian/linvclipboard_*_amd64.deb
```

---

## 🏗️ Architecture

```
  ┌─────────────────┐     ┌─────────────┐
  │  linvclip-ui    │     │   clipctl   │
  │  (Tauri v2)     │     │   (CLI)     │
  └────────┬────────┘     └──────┬──────┘
           │                     │
           └──────┬──────────────┘
                  │
        Unix Domain Socket (IPC)
                  │
           ┌──────┴──────┐
           │    clipd     │
           │   (daemon)   │
           └──────┬──────┘
                  │
     ┌────────────┼────────────┐
     │            │            │
  SQLite       FTS5        arboard
  + blobs    full-text    X11/Wayland
              search      clipboard
```

| Crate | Role |
|:------|:-----|
| **`clipd`** | Background daemon — captures clipboard changes, enforces limits, serves IPC |
| **`clipctl`** | CLI tool — list, search, paste, pin, delete, status |
| **`linvclip-ui`** | Tauri v2 overlay — the full GUI experience |
| **`shared`** | Library — database, IPC protocol, config, models |

---

## ⌨️ Usage

### Overlay UI

Press **`Super+.`** (or your custom shortcut) to summon the overlay.

| Key | Action |
|:----|:-------|
| `↑` `↓` | Navigate items |
| `Enter` | Copy selected item to clipboard |
| `Delete` | Remove selected item |
| `Escape` | Dismiss overlay |
| `Ctrl` + `+` / `-` / `0` | Zoom in / out / reset |

**Tabs:** Clipboard • Emojis • Symbols • GIFs

Just start typing to search — the search bar auto-focuses.

### CLI

```bash
clipctl list                  # Recent items
clipctl list --limit 50       # Last 50
clipctl search "hello"        # Full-text search
clipctl paste <id>            # Copy item back to clipboard
clipctl pin <id>              # Pin / unpin
clipctl delete <id>           # Delete
clipctl clear                 # Clear all non-pinned
clipctl status                # Daemon info
```

### Daemon

```bash
systemctl --user status clipd           # Status
systemctl --user restart clipd          # Restart
journalctl --user -u clipd -f           # Live logs
systemctl --user status linvclip-update-check.timer  # Update timer
```

---

## ⚙️ Configuration

Auto-created at `~/.config/linvclip/config.toml` on first run.

```toml
[daemon]
poll_interval_ms = 250
log_level = "info"                # trace | debug | info | warn | error

[storage]
max_items = 10000
max_item_size_bytes = 52428800    # 50 MB
expiry_days = 30

[security]
blacklisted_apps = ["keepassxc", "1password", "bitwarden"]
incognito = false                 # true = pause all capture

[ui]
theme = "auto"                    # auto | dark | light
language = "en"                   # en | pt | ja | hi
zoom = 100                        # 50–200
window_position = "mouse"         # mouse | fixed
```

---

## 🌐 Adding a Language

1. Copy `crates/linvclip-ui/src/i18n/en.json` → `fr.json`
2. Translate all values (keep keys unchanged)
3. Import in `crates/linvclip-ui/src/i18n/index.jsx` and add to the `TRANSLATIONS` map
4. It appears in Settings automatically

---

## 🗑️ Uninstall

```bash
sudo dpkg -r linvclipboard

# Optional: remove user data
rm -rf ~/.config/linvclip ~/.local/share/linvclip
```

---

## 🙏 Credits

This is a personal fork of the original [LinVClipBoard](https://github.com/DreamerX00/LinVClipBoard) by **[DreamerX00](https://github.com/DreamerX00)**.

Huge thanks to DreamerX00 for building such a clean, well-architected clipboard manager. The original project is the real deal — go give it a star ⭐

---

## 🔧 Changes in this fork

### Bug fixes
- **Config persistence** — settings (language, theme, etc.) no longer reset on reopen; config is now read/written directly from file instead of going through the daemon's stale in-memory state

### KDE Wayland tray
- Left-click on the tray icon now opens/closes the app instead of showing a context menu
- Uses native `ksni` SNI protocol on KDE, bypassing libappindicator which doesn't support this
- New optional setting: **"Left click opens app"** (Settings → General)

### UI / UX
- **Close-to-tray button** in the app header (✕)
- **Double-click** on any clipboard item toggles the preview pane
- Preview pane now updates on **hover** and **click** (previously only keyboard navigation updated it)
- New optional setting: **"Show OCR button"** — hide the Extract Text (OCR) button if you don't use it (Settings → Features)

### Performance
- `filteredItems` wrapped in `useMemo` — no longer recalculated on every render
- `React.memo` on clipboard list items — prevents unnecessary re-renders
- Helper functions (`formatPreview`, `formatTime`, etc.) moved outside component — no longer recreated on every render
- `fetchItems` loading guard via `useRef` instead of state — eliminates cascading hook recreations
- Fallback polling interval: 5s → 30s (push events are the primary update mechanism)
- `bulk_delete` N+1 query fixed — now uses a single `WHERE id IN (...)` instead of N separate SELECTs
- Composite SQLite index on `(pinned DESC, pin_order ASC, created_at DESC)` for the main list query
- `get_active_app_name()` tool detection cached via `OnceLock` — no more 3× fork+exec on every 250ms poll

### Localization
- Added **Russian** locale (`ru.json`)

---

## 📄 License

[MIT](LICENSE) — Original work by **DreamerX00**, fork maintained by **QA-Vlad**
