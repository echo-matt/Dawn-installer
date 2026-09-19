<script>
  let {
    isOpen = false,
    updateInfo = null,
    isUpdating = false,
    updateProgress = null,
    onClose = () => {},
    onUpdate = () => {}
  } = $props();
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="app-update-modal">
  <div class="modal-card" style="max-width: 520px; width: 92%;">
    <div class="modal-header">
      <div class="modal-title-row">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
        <h3>Launcher Update Available</h3>
      </div>
      {#if !isUpdating}
        <button class="modal-close-btn" id="app-update-close" title="Close" onclick={onClose}>&times;</button>
      {/if}
    </div>

    {#if updateInfo}
      <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 14px; background: rgba(56, 189, 248, 0.08); padding: 10px 14px; border-radius: 8px; border: 1px solid rgba(56, 189, 248, 0.2);">
        <div style="font-size: 13px; color: #94a3b8;">
          A new version of the <strong>DAWN Launcher</strong> is ready to install.
        </div>
        <div style="margin-left: auto; display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600;">
          <span style="background: rgba(255,255,255,0.08); padding: 2px 8px; border-radius: 4px; color: #94a3b8;">v{updateInfo.current_version}</span>
          <span style="color: #64748b;">➔</span>
          <span style="background: rgba(56, 189, 248, 0.2); padding: 2px 8px; border-radius: 4px; color: #38bdf8;">v{updateInfo.latest_version}</span>
        </div>
      </div>

      {#if updateInfo.release_notes}
        <div style="margin-bottom: 16px;">
          <div style="font-size: 11px; font-weight: 700; color: #94a3b8; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 6px;">
            Release Notes
          </div>
          <div style="font-size: 12px; color: #cbd5e1; line-height: 1.5; background: rgba(0,0,0,0.3); padding: 10px 12px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.06); max-height: 160px; overflow-y: auto; white-space: pre-wrap; font-family: inherit;">
            {updateInfo.release_notes}
          </div>
        </div>
      {/if}
    {/if}

    {#if isUpdating}
      <div style="margin-bottom: 16px;">
        <div style="display: flex; justify-content: space-between; font-size: 12px; color: #cbd5e1; margin-bottom: 6px;">
          <span>{updateProgress ? updateProgress.status : 'Downloading update...'}</span>
          <span style="font-weight: 600; color: #38bdf8;">{updateProgress ? updateProgress.percent : 0}%</span>
        </div>
        <div style="width: 100%; height: 6px; background: rgba(255,255,255,0.08); border-radius: 3px; overflow: hidden;">
          <div style="width: {updateProgress ? updateProgress.percent : 0}%; height: 100%; background: linear-gradient(90deg, #38bdf8, #818cf8); transition: width 0.2s ease;"></div>
        </div>
        <div style="font-size: 11px; color: #64748b; margin-top: 6px; text-align: center;">
          The launcher will restart automatically to finish installation.
        </div>
      </div>
    {/if}

    <div class="modal-actions" style="justify-content: flex-end; gap: 10px;">
      {#if !isUpdating}
        <button class="modal-btn modal-btn-cancel" type="button" onclick={onClose} style="min-width: 110px;">
          Later
        </button>
        <button class="modal-btn modal-btn-primary" type="button" onclick={onUpdate} style="min-width: 130px;">
          Update Now
        </button>
      {:else}
        <button class="modal-btn modal-btn-primary" type="button" disabled style="min-width: 150px; opacity: 0.7; cursor: wait;">
          Updating...
        </button>
      {/if}
    </div>
  </div>
</div>
