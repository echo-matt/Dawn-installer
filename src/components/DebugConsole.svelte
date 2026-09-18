<script>
  import { onMount } from 'svelte';
  import { api } from '../lib/api.js';

  let {
    isOpen = false,
    onClose = () => {}
  } = $props();

  let logs = $state([]);
  let searchQuery = $state('');
  let autoScroll = $state(true);
  let logContainer = $state(null);
  let copyFeedback = $state(false);

  async function loadLogs() {
    try {
      logs = await api.getDebugLogs();
      if (autoScroll) {
        setTimeout(scrollToBottom, 50);
      }
    } catch (err) {
      console.error('Failed to load logs:', err);
    }
  }

  function scrollToBottom() {
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  async function handleOpenLogFile() {
    try {
      await api.openLogFile();
    } catch (err) {
      console.error('Failed to open log file:', err);
    }
  }

  async function handleCopyLogs() {
    try {
      const text = filteredLogs.join('\n');
      await navigator.clipboard.writeText(text);
      copyFeedback = true;
      setTimeout(() => {
        copyFeedback = false;
      }, 2000);
    } catch (err) {
      console.error('Failed to copy logs:', err);
    }
  }

  async function handleClearLogs() {
    try {
      await api.clearDebugLogs();
      logs = [];
    } catch (err) {
      console.error('Failed to clear logs:', err);
    }
  }

  $effect(() => {
    if (isOpen) {
      loadLogs();
    }
  });

  onMount(async () => {
    const unlisten = await api.onDebugLog((line) => {
      logs = [...logs, line];
      if (autoScroll) {
        setTimeout(scrollToBottom, 30);
      }
    });

    return () => {
      if (typeof unlisten === 'function') unlisten();
    };
  });

  let filteredLogs = $derived(
    searchQuery.trim()
      ? logs.filter((l) => l.toLowerCase().includes(searchQuery.toLowerCase()))
      : logs
  );

  function getLogLevelClass(line) {
    if (line.includes('[ERROR]') || line.includes('[DEPOT_ERR]')) return 'log-error';
    if (line.includes('[WARN]')) return 'log-warn';
    if (line.includes('[INFO]')) return 'log-info';
    return 'log-depot';
  }
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="debug-console-modal">
  <div class="modal-card debug-modal-card">
    <div class="debug-header">
      <div class="debug-title-row">
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#38bdf8" stroke-width="2">
          <polyline points="4 17 10 11 4 5"></polyline>
          <line x1="12" y1="19" x2="20" y2="19"></line>
        </svg>
        <span class="debug-title">Debug Console & Logs</span>
        <span class="log-count-badge">{filteredLogs.length} entries</span>
      </div>

      <div class="debug-actions">
        <button
          type="button"
          class="debug-btn"
          title="Open log file in Notepad"
          onclick={handleOpenLogFile}
        >
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            <polyline points="15 3 21 3 21 9"></polyline>
            <line x1="10" y1="14" x2="21" y2="3"></line>
          </svg>
          <span>Open File</span>
        </button>

        <button
          type="button"
          class="debug-btn"
          title="Copy logs to clipboard"
          onclick={handleCopyLogs}
        >
          {#if copyFeedback}
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="#34d399" stroke-width="2">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
            <span style="color: #34d399;">Copied!</span>
          {:else}
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
            </svg>
            <span>Copy</span>
          {/if}
        </button>

        <button
          type="button"
          class="debug-btn debug-btn-danger"
          title="Clear logs"
          onclick={handleClearLogs}
        >
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
          <span>Clear</span>
        </button>

        <button
          type="button"
          class="modal-close-btn"
          title="Close Debug Console"
          onclick={onClose}
        >
          &times;
        </button>
      </div>
    </div>

    <!-- Search / Filter Bar -->
    <div class="debug-filter-bar">
      <div class="search-input-wrapper">
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          class="debug-search-input"
          placeholder="Filter logs (e.g. error, Steam, manifest, depot)..."
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="clear-search-btn" onclick={() => (searchQuery = '')}>&times;</button>
        {/if}
      </div>

      <label class="autoscroll-toggle">
        <input type="checkbox" bind:checked={autoScroll} />
        <span>Auto-scroll</span>
      </label>
    </div>

    <!-- Log Viewer Content -->
    <div class="debug-log-view" bind:this={logContainer}>
      {#if filteredLogs.length === 0}
        <div class="empty-logs">
          {#if searchQuery}
            No logs matching "{searchQuery}"
          {:else}
            No logs recorded yet. Action and DepotDownloader messages will appear here.
          {/if}
        </div>
      {:else}
        {#each filteredLogs as line, i (i)}
          <div class="log-line {getLogLevelClass(line)}">
            <span class="log-text">{line}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .debug-modal-card {
    width: 760px;
    max-width: 92vw;
    height: 520px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    padding: 16px;
    background: rgba(15, 23, 42, 0.95);
    border: 1px solid rgba(56, 189, 248, 0.3);
    border-radius: 12px;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7), 0 0 30px rgba(56, 189, 248, 0.15);
    backdrop-filter: blur(16px);
  }

  .debug-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .debug-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .debug-title {
    font-size: 14px;
    font-weight: 600;
    color: #f1f5f9;
    letter-spacing: 0.02em;
  }

  .log-count-badge {
    font-size: 11px;
    background: rgba(255, 255, 255, 0.08);
    color: #94a3b8;
    padding: 2px 7px;
    border-radius: 10px;
    font-family: ui-monospace, monospace;
  }

  .debug-actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .debug-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    color: #cbd5e1;
    font-size: 11px;
    font-weight: 500;
    padding: 5px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .debug-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: #f8fafc;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .debug-btn-danger:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .debug-filter-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 0;
    gap: 12px;
  }

  .search-input-wrapper {
    position: relative;
    flex: 1;
    display: flex;
    align-items: center;
  }

  .search-input-wrapper svg {
    position: absolute;
    left: 10px;
    color: #64748b;
    pointer-events: none;
  }

  .debug-search-input {
    width: 100%;
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 30px 6px 32px;
    color: #e2e8f0;
    font-size: 12px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s ease;
  }

  .debug-search-input:focus {
    border-color: #38bdf8;
    background: rgba(0, 0, 0, 0.45);
  }

  .clear-search-btn {
    position: absolute;
    right: 8px;
    background: transparent;
    border: none;
    color: #94a3b8;
    font-size: 15px;
    cursor: pointer;
    padding: 0 4px;
  }

  .autoscroll-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: #94a3b8;
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  .autoscroll-toggle input {
    cursor: pointer;
    accent-color: #38bdf8;
  }

  .debug-log-view {
    flex: 1;
    overflow-y: auto;
    background: #090d16;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    padding: 10px 12px;
    font-family: 'Cascadia Code', 'Fira Code', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 11px;
    line-height: 1.5;
    user-select: text;
  }

  .empty-logs {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #64748b;
    font-style: italic;
  }

  .log-line {
    word-break: break-all;
    white-space: pre-wrap;
    padding: 1px 0;
  }

  .log-error {
    color: #f87171;
    background: rgba(239, 68, 68, 0.08);
    border-left: 2px solid #ef4444;
    padding-left: 6px;
  }

  .log-warn {
    color: #fbbf24;
  }

  .log-info {
    color: #38bdf8;
  }

  .log-depot {
    color: #94a3b8;
  }
</style>