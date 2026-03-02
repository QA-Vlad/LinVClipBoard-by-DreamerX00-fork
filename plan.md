# LinVClipBoard v2.0 — Major UI & Feature Overhaul Plan

> **Goal:** Transform LinVClipBoard into a world-class clipboard manager with an Emoji/Symbol picker,
> multi-tab interface, i18n, window positioning, zoom control, and a redesigned Settings panel —
> inspired by [Linux-ClipBoard](https://github.com/bruno33223/Linux-ClipBoard).
>
> **Target version:** `v2.0.0`
> **Start:** 2026-03-02

---

## Architecture Overview

```
┌──────────────────────────────────────────────────┐
│  App Shell (app-header + tab-bar + content)      │
│ ┌──────────────────────────────────────────────┐ │
│ │  Header:  🗒️ LinVClipBoard      [⚙️] [🗑️]   │ │
│ ├──────────────────────────────────────────────┤ │
│ │  TabBar:  📋 Clipboard  😀 Emojis  Σ Symbols│ │
│ ├──────────────────────────────────────────────┤ │
│ │  SearchBar                                   │ │
│ ├──────────────────────────────────────────────┤ │
│ │  FilterPills (Clipboard tab only):           │ │
│ │    [All] [Text] [Images] [Files] [Pinned]    │ │
│ ├──────────────────────────────────────────────┤ │
│ │  Content Area (tab-dependent):               │ │
│ │    • Clipboard → ClipboardList               │ │
│ │    • Emojis    → EmojiPicker                 │ │
│ │    • Symbols   → SymbolPicker                │ │
│ ├──────────────────────────────────────────────┤ │
│ │  Footer:  "Crafted by akash-singh"           │ │
│ └──────────────────────────────────────────────┘ │
│                                                  │
│  [Settings Panel – overlay]                      │
│  [Confirm Dialog – overlay]                      │
└──────────────────────────────────────────────────┘
```

### New File Map

```
crates/linvclip-ui/src/
├── App.jsx                    ← rewrite: tab system, i18n context, zoom
├── main.jsx                   ← unchanged
├── styles.css                 ← massive expansion
├── i18n/
│   ├── index.js               ← i18n provider + hook
│   ├── en.json                ← English strings
│   └── pt.json                ← Portuguese strings
├── data/
│   ├── emojis.json            ← curated emoji dataset (categorized)
│   └── symbols.json           ← math/special symbols dataset
├── components/
│   ├── AppHeader.jsx          ← NEW: title bar with ⚙️ 🗑️ buttons
│   ├── TabBar.jsx             ← NEW: Clipboard / Emojis / Symbols
│   ├── SearchBar.jsx          ← enhance: works across all tabs
│   ├── FilterPills.jsx        ← NEW: All / Text / Images / Files / Pinned
│   ├── ClipboardList.jsx      ← enhance: timestamp format, drag dots
│   ├── EmojiPicker.jsx        ← NEW: grid with categories, skin tones
│   ├── SymbolPicker.jsx       ← NEW: math, arrows, currency, etc.
│   ├── SettingsPanel.jsx      ← rewrite: language, position, zoom, shortcuts info
│   ├── ConfirmDialog.jsx      ← minor polish
│   └── StatusBar.jsx          ← replace with Footer.jsx
├── hooks/
│   └── useZoom.js             ← NEW: zoom state + CSS variable setter
```

---

## Phase 1 — Internationalization (i18n) Foundation
> All user-visible strings go through `t()`. Two languages: English + Portuguese.

- [x] **1.1** Create `src/i18n/en.json` — all English strings (header, tabs, search placeholder, settings labels, filter pills, footer, empty states, toasts)
- [x] **1.2** Create `src/i18n/pt.json` — Portuguese translation
- [x] **1.3** Create `src/i18n/index.js` — React context provider + `useTranslation()` hook + `<I18nProvider>` wrapper
- [x] **1.4** Wrap `<App>` in `<I18nProvider>` inside `main.jsx`
- [x] **1.5** Add `language` field to `UiConfig` in `shared/src/config.rs` with `#[serde(default)]` defaulting to `"en"`
- [x] **1.6** Update `SettingsPanel` to persist language choice to backend config

---

## Phase 2 — App Shell Redesign (Header + Tabs + Footer)
> Replace flat layout with header → tab bar → content → footer.

- [x] **2.1** Create `AppHeader.jsx` — app icon + title "LinVClipBoard", settings gear button (⚙️), clear-all trash button (🗑️) with confirmation
- [x] **2.2** Create `TabBar.jsx` — three tabs: `📋 Clipboard`, `😀 Emojis`, `Σ Symbols` with active state indicator underline; all labels run through `t()`
- [x] **2.3** Create `Footer.jsx` — "Crafted by akash-singh" or "powered by akash-singh" with the current language; replaces old StatusBar
- [x] **2.4** Rewrite `App.jsx`:
  - Add `activeTab` state (`"clipboard" | "emojis" | "symbols"`)
  - Render: `AppHeader` → `TabBar` → conditionally `SearchBar` → tab content → `Footer`
  - Pass settings/clear handlers to header
  - Remove old StatusBar import/usage
- [x] **2.5** Add all supporting CSS: `.app-header`, `.tab-bar`, `.tab-btn`, `.tab-active-indicator`, `.footer`
- [x] **2.6** Remove old `StatusBar.jsx` or keep as dead code (prefer remove)

---

## Phase 3 — Filter Pills (Clipboard Tab)
> Type-based filtering: All / Text / Images / Files / Pinned

- [x] **3.1** Create `FilterPills.jsx` — horizontal pill row; active pill highlighted in accent color; emits `onFilterChange(filter)`
- [x] **3.2** Add `filterType` state in `App.jsx`: `"all" | "text" | "image" | "files" | "pinned"`
- [x] **3.3** Client-side filtering: filter `items` array before passing to `ClipboardList`
- [ ] **3.4** (Optional) Backend support: add `IpcRequest::ListFiltered { content_type, offset, limit }` for server-side filtering
- [x] **3.5** CSS: `.filter-pills`, `.pill`, `.pill.active` with smooth transitions
- [x] **3.6** Show filter pills only when `activeTab === "clipboard"`

---

## Phase 4 — Clipboard List Enhancement
> Better item cards matching the reference UI.

- [x] **4.1** Add drag-handle dots (⠿) on the left side of each item for visual grip
- [x] **4.2** Show full `HH:MM:SS` timestamp instead of relative "5m ago"
- [x] **4.3** Improve image preview: show actual thumbnail inline (already have `get_image_base64`)
- [x] **4.4** Show content type label below preview (e.g., "clipboard-image.png" for images)
- [x] **4.5** Better text truncation: monospace font for code, normal for text
- [x] **4.6** Update item card CSS for tighter layout matching reference screenshots
- [x] **4.7** Run all static strings through `t()` (empty state, etc.)

---

## Phase 5 — Emoji Picker Tab
> Searchable emoji grid with categories.

- [x] **5.1** Create `src/data/emojis.json` — curated emoji dataset organized by category: Common, Smileys, People, Animals, Food, Travel, Activities, Objects, Symbols, Flags (use Unicode 15.0)
- [x] **5.2** Create `EmojiPicker.jsx`:
  - Search/filter emojis by name/keyword
  - Category headers ("COMMON", "SMILEYS & PEOPLE", etc.)
  - Grid layout (8-10 columns)
  - Click → copy to system clipboard via `invoke("paste_raw_text", { text: emoji })`
  - Show toast on copy
  - Recently-used section at top (persisted in localStorage)
- [x] **5.3** Add Tauri command `paste_raw_text` in `lib.rs` — writes arbitrary text to clipboard (for emoji/symbol insertion)
- [x] **5.4** CSS: `.emoji-grid`, `.emoji-cell`, `.emoji-category-header`, hover/active states
- [x] **5.5** SearchBar integration: when on Emojis tab, search filters emojis by name
- [ ] **5.6** Skin tone selector (optional stretch goal)

---

## Phase 6 — Symbol Picker Tab
> Math, arrows, currency, and special characters.

- [x] **6.1** Create `src/data/symbols.json` — categorized: Math (±, ÷, ∞, √, π, ∑), Arrows (→, ←, ↑, ↓, ⇒), Currency ($, €, £, ¥, ₹, ₿), Greek (α, β, γ), Superscript/Subscript, Punctuation, Box Drawing
- [x] **6.2** Create `SymbolPicker.jsx` — same grid pattern as EmojiPicker, category headers, search, click-to-copy
- [x] **6.3** CSS: `.symbol-grid`, `.symbol-cell` (slightly wider cells for multi-char symbols)
- [x] **6.4** SearchBar integration: filter by symbol name ("infinity" → ∞)
- [x] **6.5** Recently-used section (localStorage)

---

## Phase 7 — Settings Panel Redesign
> Match the reference design: Language, Window Position, Size/Zoom, Keyboard Shortcuts.

- [x] **7.1** Redesign layout with icon-labeled sections instead of `<fieldset>`:
  - 🌐 Language — toggle buttons: Português / English
  - 🖥️ Window Position — toggle buttons: Fixed / Mouse
  - 🔍 Size (Zoom) — slider 50%–200% with percentage label
  - ⌨️ Keyboard Shortcuts — informational panel (instructions for creating system shortcuts)
- [x] **7.2** Add `window_position` field to `UiConfig` in `config.rs`: `"fixed" | "mouse"` (default: `"mouse"`)
- [x] **7.3** Add `zoom` field (u32, percent) to `UiConfig` (default: `100`)
- [x] **7.4** Implement zoom via CSS `transform: scale()` or `font-size` on `:root`
- [x] **7.5** Implement window positioning logic in Tauri `lib.rs`:
  - `"fixed"` → always center on primary monitor
  - `"mouse"` → spawn at cursor position (existing `center_on_active_monitor` enhanced)
- [x] **7.6** Auto-save on close: "Close to save and apply" behavior instead of Save/Cancel buttons
- [x] **7.7** i18n: all settings labels through `t()`
- [x] **7.8** Move Daemon/Storage/Security settings to an "Advanced" collapsible section

---

## Phase 8 — Zoom System
> User can scale the entire UI from 50% to 200%.

- [x] **8.1** Create `src/hooks/useZoom.js` — reads zoom from config, applies `document.documentElement.style.fontSize` or CSS custom property
- [x] **8.2** Persist zoom in `UiConfig.zoom` via backend config
- [x] **8.3** Settings slider updates zoom in real-time (live preview)
- [x] **8.4** Keyboard shortcuts: `Ctrl++` / `Ctrl+-` / `Ctrl+0` for zoom in/out/reset
- [x] **8.5** Ensure all UI components scale properly (use `rem`/`em` units, not `px` where possible)

---

## Phase 9 — Window Position Logic
> "Fixed" (center of screen) vs "Mouse" (near cursor).

- [x] **9.1** Read `ui.window_position` from config in Tauri `run()` setup
- [x] **9.2** `"mouse"` mode: position window near cursor with screen-edge clamping (don't go off-screen)
- [x] **9.3** `"fixed"` mode: center on primary monitor (current `center_on_active_monitor`)
- [x] **9.4** Apply on every show (shortcut press, tray click)
- [x] **9.5** Update `center_on_active_monitor` → generalize to `position_window(window, mode)`

---

## Phase 10 — Tauri Backend Additions
> New commands needed by the UI features.

- [x] **10.1** `paste_raw_text` command — write arbitrary text (emoji/symbol) directly to system clipboard, then simulate Ctrl+V or just set clipboard
- [x] **10.2** `get_config` / `save_config` — already exist, just ensure `language`, `zoom`, `window_position` fields survive round-trip
- [ ] **10.3** Update Tauri window config: make window resizable within bounds for zoom
- [ ] **10.4** Add `window.set_size()` call when zoom changes to adjust window dimensions
- [ ] **10.5** Persist recently-used emojis/symbols: could use localStorage (simpler) or a new Tauri command

---

## Phase 11 — CSS Overhaul & Polish
> Comprehensive style update to match the reference design.

- [ ] **11.1** App header styles: draggable region, icon placement, button spacing
- [ ] **11.2** Tab bar styles: underline active indicator, icon+label layout, hover effects
- [ ] **11.3** Filter pill styles: rounded pills, accent fill on active, smooth transitions
- [ ] **11.4** Emoji/Symbol grid: uniform cell size, hover scale effect, smooth scroll
- [ ] **11.5** Settings panel rewrite: section icons, toggle buttons (not dropdowns for lang/position), range slider for zoom
- [ ] **11.6** Footer: subtle text, centered, separator line above
- [ ] **11.7** Clipboard item cards: left grip dots, tighter padding, better timestamp display
- [ ] **11.8** Ensure light theme works for all new components
- [ ] **11.9** Smooth animations: tab switch, panel open/close, pill select
- [ ] **11.10** Responsive scaling with zoom system

---

## Phase 12 — Integration & Testing
> Wire everything together, fix edge cases.

- [ ] **12.1** Verify all three tabs work: switch, search, content rendering
- [ ] **12.2** Verify i18n: switch language, all strings update, persists on restart
- [ ] **12.3** Verify settings: change language/position/zoom, close panel, reopen — values persist
- [ ] **12.4** Verify zoom: slider in settings changes UI scale, keyboard shortcuts work
- [ ] **12.5** Verify window positioning: Fixed vs Mouse modes both work on multi-monitor
- [ ] **12.6** Verify emoji/symbol copy: click emoji → copied to clipboard → toast shown
- [ ] **12.7** Keyboard navigation: Tab/Shift+Tab cycles tabs, arrow keys navigate within tab content
- [ ] **12.8** `cargo check --workspace` passes with zero errors
- [ ] **12.9** `cargo clippy --workspace` passes with zero warnings
- [ ] **12.10** Version bump to `2.0.0` everywhere (Cargo.toml, package.json, tauri.conf.json)

---

## Phase 13 — Cleanup & Documentation
> Final polish before merge.

- [ ] **13.1** Remove dead `StatusBar.jsx` (replaced by `Footer.jsx` + header buttons)
- [ ] **13.2** Update `README.md` with new screenshots and feature list
- [ ] **13.3** Update `.desktop` file description
- [ ] **13.4** Add contributing guide for i18n (how to add new languages)
- [ ] **13.5** Final git commit: `v2.0.0 — Major UI overhaul`

---

## Summary Table

| Phase | Description | Items | Status |
|-------|-------------|-------|--------|
| 1 | i18n Foundation | 6 | ⬜ Not started |
| 2 | App Shell (Header + Tabs + Footer) | 6 | ⬜ Not started |
| 3 | Filter Pills | 6 | ⬜ Not started |
| 4 | Clipboard List Enhancement | 7 | ⬜ Not started |
| 5 | Emoji Picker | 6 | ⬜ Not started |
| 6 | Symbol Picker | 5 | ⬜ Not started |
| 7 | Settings Panel Redesign | 8 | ⬜ Not started |
| 8 | Zoom System | 5 | ⬜ Not started |
| 9 | Window Position Logic | 5 | ⬜ Not started |
| 10 | Tauri Backend Additions | 5 | ⬜ Not started |
| 11 | CSS Overhaul & Polish | 10 | ⬜ Not started |
| 12 | Integration & Testing | 10 | ⬜ Not started |
| 13 | Cleanup & Documentation | 5 | ⬜ Not started |
| **Total** | | **84** | |

---

## Dependency Graph

```
Phase 1 (i18n) ──────────────────────────┐
Phase 7.2-7.3 (config fields) ───────────┤
Phase 10.1 (paste_raw_text cmd) ──────┐  │
                                      │  │
Phase 2 (App Shell) ◄────────────────────┘
    │                                 │
    ├── Phase 3 (Filter Pills)        │
    ├── Phase 4 (Clipboard Enhance)   │
    ├── Phase 5 (Emoji Picker) ◄──────┘
    ├── Phase 6 (Symbol Picker) ◄─────┘
    │
Phase 7 (Settings Redesign) ◄── Phase 1
Phase 8 (Zoom) ◄── Phase 7
Phase 9 (Window Position) ◄── Phase 7
Phase 10 (Backend) ◄── parallel with phases 5-6
Phase 11 (CSS) ◄── after phases 2-9
Phase 12 (Testing) ◄── after all
Phase 13 (Cleanup) ◄── after phase 12
```

### Recommended Execution Order
1. **Phase 1** (i18n) + **Phase 10** (backend) — foundations, no UI deps
2. **Phase 2** (shell) + **Phase 7.2–7.3** (config fields) — structural
3. **Phase 3** (filters) + **Phase 4** (list enhance) — clipboard tab
4. **Phase 5** (emoji) + **Phase 6** (symbols) — new tabs
5. **Phase 7** (settings full) + **Phase 8** (zoom) + **Phase 9** (position)
6. **Phase 11** (CSS polish) — visual consistency
7. **Phase 12** (testing) → **Phase 13** (cleanup)
