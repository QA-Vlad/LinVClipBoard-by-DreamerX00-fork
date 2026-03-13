# LinVClipBoard — UX & Tray Modifications Plan

> **Version**: v2.3.0 → v2.3.1
> **Scope**: 4 targeted fixes/enhancements
> **Files affected**: `lib.rs`, `config.rs`, `SettingsPanel.jsx`, `App.jsx`, i18n files

---

## Issues to Fix

### 1. Configurable Tray Item Count

**Problem**: Tray menu always shows 5 latest items, hardcoded. No setting to increase/decrease.

**Current code**: `lib.rs:1373-1376` — `IpcRequest::List { offset: 0, limit: 5 }`

**Fix**:
- Add `tray_items: u32` field to `UiConfig` in `config.rs` (default: `5`, range: `3–15`)
- Read `config.ui.tray_items` in `refresh_tray_menu()` instead of hardcoded `5`
- Add slider in Settings → General section in `SettingsPanel.jsx`
- Add i18n key: `settings.tray_items` ("Tray menu items")

**Files to modify**:
- `crates/shared/src/config.rs` — add `tray_items` field + default
- `crates/linvclip-ui/src-tauri/src/lib.rs` — use config value in `refresh_tray_menu()`
- `crates/linvclip-ui/src/components/SettingsPanel.jsx` — add slider UI
- i18n: `en.json`, `pt.json`, `hin.json`, `japanese.json`

---

### 2. Tray Click: Silent Paste (No Window Flash)

**Problem**: Clicking a tray menu item opens the app window briefly (600ms flash), shows "copied" animation, then hides. This is unnecessary — the user already knows they clicked the item.

**Current code**: `lib.rs:1527-1531` — shows window, waits 600ms, hides.

**Fix**:
- Remove the show/focus/sleep/hide block from tray paste handler
- Just do the paste via IPC (the item is already on clipboard)
- After IPC paste, simulate Ctrl+V into the previously-focused window using `try_paste_shortcut()` (same mechanism as `type_text()`)
- No window shown, no animation

**Files to modify**:
- `crates/linvclip-ui/src-tauri/src/lib.rs` — remove window flash in `paste_*` handler, add paste simulation

---

### 3. Window Positioning at Mouse Cursor (Multi-Monitor/Workspace)

**Problem**: The overlay sometimes doesn't appear on the active monitor/workspace. It uses `window.cursor_position()` which works on X11 but has Wayland limitations.

**Current code**: `lib.rs:1322-1365` — `position_window()` gets cursor position, finds monitor, positions window.

**How Windows Win+V works**:
- Appears near the text caret (input cursor) or mouse pointer
- Always shows on the active monitor/workspace
- Does NOT steal focus from the input field — it's a floating overlay
- Clicking an item auto-pastes into the previously focused input and closes the panel

**Fix**:
- Keep the existing `position_window()` logic (already uses cursor_position + monitor detection)
- Add a pre-positioning step: before showing, get cursor position and validate it's within a known monitor's bounds. If not (Wayland edge case), fall back to centering on primary monitor
- Add `window.set_always_on_top(true)` to ensure it shows above all windows on any workspace
- The `alwaysOnTop: true` is already set in `tauri.conf.json` — verify it applies to all DEs

**Files to modify**:
- `crates/linvclip-ui/src-tauri/src/lib.rs` — harden `position_window()` for edge cases

---

### 4. Auto-Paste on Item Select + Close Overlay (Win+V Behavior)

**Problem**: When user opens clipboard overlay from an input field and clicks an item, the text is copied to clipboard but user must manually Ctrl+V. The overlay stays open.

**How Windows Win+V works**:
- User is in an input field, presses Win+V
- Clipboard panel appears (floating, doesn't steal focus)
- User clicks an item → it's instantly pasted into the input field
- Panel closes automatically

**Fix — "Paste & Close" flow**:

1. **On shortcut press (before showing overlay)**:
   - Record the currently focused window ID using `xdotool getactivewindow` (X11) or track it from the compositor
   - Store as app state: `previous_window_id`

2. **On item click in overlay** (frontend → Tauri command):
   - Copy selected text to system clipboard
   - Hide the overlay window
   - Wait 80ms for compositor to return focus to previous window
   - Simulate Ctrl+V using `try_paste_shortcut()` (wtype for Wayland, xdotool for X11)
   - Alternatively, re-focus the previous window first: `xdotool windowactivate {id}` then simulate paste

3. **New Tauri command**: `paste_and_close(id: String)`
   - Copies item to clipboard
   - Hides window
   - Restores focus to previous window
   - Simulates Ctrl+V
   - This replaces the current `paste_item` behavior when overlay is invoked via shortcut

4. **Frontend change** (`App.jsx`):
   - Item click handler: call `invoke("paste_and_close", { id })` instead of `paste_item`
   - The overlay closes immediately on click

**Auto-paste tool chain** (already exists in codebase):
- **X11**: `xdotool key --clearmodifiers ctrl+v`
- **Wayland (wlroots)**: `wtype -M ctrl -P v -p v -m ctrl`
- **Fallback**: Text stays on clipboard for manual Ctrl+V

**Files to modify**:
- `crates/linvclip-ui/src-tauri/src/lib.rs`:
  - New state: `previous_window_id: Mutex<Option<String>>`
  - Capture previous window on shortcut press (before show)
  - New command: `paste_and_close()`
- `crates/linvclip-ui/src/App.jsx` — item click calls `paste_and_close`
- `crates/linvclip-ui/src/components/ClipboardList.jsx` — if paste callback changes

---

## Implementation Order

1. **Fix 2** (tray silent paste) — smallest change, immediate UX improvement
2. **Fix 1** (configurable tray items) — config + settings UI
3. **Fix 4** (auto-paste + close) — core Win+V behavior, biggest impact
4. **Fix 3** (multi-monitor hardening) — defensive improvement

---

## i18n Keys to Add

| Key | English | Portuguese | Hindi | Japanese |
|-----|---------|------------|-------|----------|
| `settings.tray_items` | Tray menu items | Itens do menu da bandeja | ट्रे मेनू आइटम | トレイメニュー項目 |
| `settings.tray_items_desc` | Number of recent items in system tray | Número de itens recentes na bandeja | सिस्टम ट्रे में हाल के आइटम | トレイに表示する最近の項目数 |
| `settings.auto_paste` | Auto-paste on select | Colar ao selecionar | चयन पर ऑटो पेस्ट | 選択時に自動ペースト |
| `settings.auto_paste_desc` | Paste and close when clicking an item | Colar e fechar ao clicar | आइटम क्लिक करने पर पेस्ट करें और बंद करें | アイテムをクリックで貼り付けて閉じる |

---

## Testing Checklist

- [ ] Tray shows configurable number of items (change in settings, verify after 5s refresh)
- [ ] Tray item click: pastes silently, no window flash
- [ ] Open overlay on monitor 1, verify position near cursor
- [ ] Open overlay on monitor 2 / different workspace, verify it follows
- [ ] From a text editor: Super+. → click item → text is pasted into editor, overlay closes
- [ ] From a terminal: Super+. → click item → text is pasted, overlay closes
- [ ] From a browser input field: same test
- [ ] No input focused: Super+. → click item → text on clipboard, overlay closes, no crash
- [ ] Settings: tray items slider works (3–15 range)
- [ ] All 4 languages display new strings correctly

---

## Version Bump

- Bump to `v2.3.1` (patch release — UX improvements, no new features)
