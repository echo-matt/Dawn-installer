<script>
  let {
    isOpen = false,
    gameFolder = '',
    onClose = () => {},
    onConfirm = () => {}
  } = $props();
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="uninstall-confirm-modal">
  <div class="modal-card">
    <div class="modal-header">
      <div class="modal-title-row">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#ef4444" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          <line x1="10" y1="11" x2="10" y2="17"></line>
          <line x1="14" y1="11" x2="14" y2="17"></line>
        </svg>
        <h3>Uninstall Dawn Mod</h3>
      </div>
      <button class="modal-close-btn" id="uninstall-modal-close" title="Cancel" onclick={onClose}>&times;</button>
    </div>

    <p class="modal-desc">
      Are you sure you want to completely uninstall Dawn from:
    </p>

    <div class="dest-preview" style="margin-bottom: 14px; word-break: break-all; font-family: monospace; font-size: 11px; background: rgba(0,0,0,0.3); padding: 8px 12px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.06); color: #cbd5e1;">
      {gameFolder || 'Selected game folder'}
    </div>

    <div style="font-size: 11px; color: #94a3b8; line-height: 1.6; margin-bottom: 18px;">
      <div style="display: flex; align-items: center; gap: 6px; margin-bottom: 4px;">
        <span style="color: #ef4444;">•</span> Deletes Dawn mod directories, scripts, presets, and configs
      </div>
      <div style="display: flex; align-items: center; gap: 6px; margin-bottom: 4px;">
        <span style="color: #ef4444;">•</span> Purges <code>.dawn</code> backups, journals, and cache files
      </div>
      <div style="display: flex; align-items: center; gap: 6px;">
        <span style="color: #38bdf8;">•</span> Restores original <code>steam_api64.dll</code> from backup
      </div>
    </div>

    <div class="modal-actions">
      <button class="modal-btn modal-btn-cancel" id="uninstall-modal-cancel" type="button" onclick={onClose}>
        Cancel
      </button>
      <button class="modal-btn modal-btn-danger" id="uninstall-modal-confirm" type="button" onclick={onConfirm}>
        Uninstall Dawn
      </button>
    </div>
  </div>
</div>
