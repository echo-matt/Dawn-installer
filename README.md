# DAWN Installer Client

A modern desktop installer client for **Dawn** (Destiny 2 Build 86657), featuring the custom animated dawn gradient background, frameless window styling, and seamless installation tools.

## Features

- **Custom Dawn Brand Artwork**: Features the official Dawn logo (`dawn_logo5.png`) with ambient breathing illumination and the subtitle "Destiny 2 Build 86657 • Installer Client".
- **Streamlined Minimalist Titlebar**: Removed the user profile avatar to present a clean, distraction-free installer window with language selection ("EN") and custom minimize/close controls.
- **Ambient Animated Wavy Gradient**: Soft, dimmed-down dawn atmospheric shader with gentle, rhythmic sinusoidal wave swells and subtle vignette framing.
- **Game Detection & Validation**:
  - Automatically scans drives and common Steam locations for `destiny2.exe` and `packages/`.
  - Interactive path selector pill with real-time validation indicator dot (green for ready, orange for pending).
  - Native Windows folder picker integration via Electron.
- **Installation Engine for `D:\Documents\VSCode stuff\Dawn`**:
  - Real-time progress bar and deployment status.
  - Expandable live console log viewing file copies, backups, and script output.
  - Automatically deploys Lua mission scripts, default configuration profiles (`settings.json`, `hud.json`, `movement.json`, `player.json`), vendor rules, event presets, and launch scripts.
  - Rollback / Restore support: easily undo changes from previous backups under `.dawn\backup`.
  - Cache clearing utility: easily remove stale `.bin` files from the Dawn cache.
  - One-click game launch with `DAWN_FOREST_BASELINE=1`.

## How to Run

### Native Desktop Application (Electron)
```bash
npm start
```

### Browser Development Preview (Vite)
```bash
npm run dev
```

### Production Build
```bash
npm run build
```
