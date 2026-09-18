<script>
  let {
    isOpen = false,
    onClose = () => {}
  } = $props();
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="linux-launch-modal">
  <div class="modal-card">
    <div class="modal-header">
      <div class="modal-title-row">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#f59e0b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
        <h3>Linux Launch Not Supported</h3>
      </div>
      <button class="modal-close-btn" id="linux-modal-close" title="Close" onclick={onClose}>&times;</button>
    </div>

    <p class="modal-desc" style="font-size: 13px; line-height: 1.5; color: #e2e8f0; margin-bottom: 14px;">
      Direct game launch from the Dawn Launcher is not supported on Linux.
    </p>

    <div style="font-size: 12px; color: #94a3b8; line-height: 1.6; margin-bottom: 20px; background: rgba(0,0,0,0.3); padding: 12px 14px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.06);">
      <div style="margin-bottom: 8px; color: #cbd5e1; font-weight: 600;">How to play your installation:</div>
      <div style="display: flex; align-items: flex-start; gap: 8px; margin-bottom: 6px;">
        <span style="color: #38bdf8;">•</span>
        <span>Run the installed game on a <strong>Windows</strong> system or partition.</span>
      </div>
      <div style="display: flex; align-items: flex-start; gap: 8px;">
        <span style="color: #38bdf8;">•</span>
        <span>Alternatively, configure and launch <code>destiny2.exe</code> using your preferred Wine or Proton runner (e.g. <strong>Lutris</strong>, <strong>Heroic</strong>, <strong>Bottles</strong>, or as a Non-Steam Game in Steam).</span>
      </div>
    </div>

    <div class="modal-actions" style="justify-content: flex-end;">
      <button class="modal-btn modal-btn-cancel" id="linux-modal-ok" type="button" onclick={onClose} style="min-width: 100px;">
        Understood
      </button>
    </div>
  </div>
</div>
