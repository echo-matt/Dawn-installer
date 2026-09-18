<script>
  import { api } from '../lib/api.js';

  let {
    onOpenDebugLogs = () => {}
  } = $props();

  function handleMinimize() {
    api.minimizeWindow();
  }

  function handleClose() {
    api.closeWindow();
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar-brand" data-tauri-drag-region>
    <span class="brand-text" data-tauri-drag-region>DAWN</span>
  </div>

  <div class="titlebar-controls">
    <button
      type="button"
      class="titlebar-tool-btn"
      id="titlebar-logs-btn"
      title="Open Debug Logs"
      onclick={onOpenDebugLogs}
    >
      <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="4 17 10 11 4 5"></polyline>
        <line x1="12" y1="19" x2="20" y2="19"></line>
      </svg>
      <span>Debug Logs</span>
    </button>

    <div class="window-buttons">
      <button class="win-btn" id="win-min" title="Minimize" onclick={handleMinimize}>
        <svg width="11" height="1" viewBox="0 0 11 1">
          <rect width="11" height="1" fill="currentColor" />
        </svg>
      </button>
      <button class="win-btn close-btn" id="win-close" title="Close" onclick={handleClose}>
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M 0 0 L 10 10 M 10 0 L 0 10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</header>

<style>
  .titlebar-tool-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #94a3b8;
    font-size: 11px;
    font-weight: 500;
    padding: 4px 9px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
    -webkit-app-region: no-drag;
  }

  .titlebar-tool-btn:hover {
    background: rgba(56, 189, 248, 0.12);
    border-color: rgba(56, 189, 248, 0.3);
    color: #38bdf8;
  }
</style>
