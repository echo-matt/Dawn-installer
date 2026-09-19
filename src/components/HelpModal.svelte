<script>
  let {
    isOpen = false,
    onClose = () => {}
  } = $props();
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="help-modal">
  <div class="modal-card help-modal-card" style="max-width: 580px; width: 94%; max-height: 88vh; display: flex; flex-direction: column;">
    <div class="modal-header" style="padding-bottom: 12px; border-bottom: 1px solid rgba(255,255,255,0.08);">
      <div class="modal-title-row" style="display: flex; align-items: center; gap: 10px;">
        <div style="background: rgba(56, 189, 248, 0.15); width: 32px; height: 32px; border-radius: 8px; display: flex; align-items: center; justify-content: center; border: 1px solid rgba(56, 189, 248, 0.3);">
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
        </div>
        <div>
          <h3 style="margin: 0; font-size: 16px; font-weight: 600; color: #f8fafc;">How to Install & Play Destiny 2 + Dawn</h3>
          <div style="font-size: 12px; color: #94a3b8; margin-top: 2px;">Quick setup guide & common troubleshooting</div>
        </div>
      </div>
      <button class="modal-close-btn" id="help-modal-close" title="Close" onclick={onClose}>&times;</button>
    </div>

    <div class="help-content-scroll" style="overflow-y: auto; padding: 16px 2px 8px 0; display: flex; flex-direction: column; gap: 14px;">
      
      <!-- 1. Folder Selection Rules -->
      <div class="help-card" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px; padding: 14px;">
        <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 13px; color: #38bdf8; margin-bottom: 8px;">
          <span style="background: #38bdf8; color: #0f172a; width: 20px; height: 20px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700;">1</span>
          Choose an EMPTY Folder for the Game
        </div>
        <p style="font-size: 12.5px; color: #cbd5e1; line-height: 1.5; margin: 0 0 10px 0;">
          When the installer asks for a destination folder, select a fresh, dedicated folder (for example: <code style="background: rgba(255,255,255,0.08); padding: 2px 6px; border-radius: 4px; color: #7dd3fc;">C:\Games\Dawn</code> or <code style="background: rgba(255,255,255,0.08); padding: 2px 6px; border-radius: 4px; color: #7dd3fc;">D:\Destiny2-Dawn</code>).
        </p>

        <div style="display: flex; flex-direction: column; gap: 6px; font-size: 12px; background: rgba(239, 68, 68, 0.08); border: 1px solid rgba(239, 68, 68, 0.2); border-radius: 8px; padding: 10px 12px;">
          <div style="font-weight: 700; color: #f87171; font-size: 11px; text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 2px;">
            Critical Rules:
          </div>
          <div style="display: flex; align-items: flex-start; gap: 8px; color: #fca5a5;">
            <span style="color: #ef4444; font-weight: bold;">✕</span>
            <span><strong>Do NOT</strong> choose your current retail Destiny 2 folder.</span>
          </div>
          <div style="display: flex; align-items: flex-start; gap: 8px; color: #fca5a5;">
            <span style="color: #ef4444; font-weight: bold;">✕</span>
            <span><strong>Do NOT</strong> choose a OneDrive or cloud-synced folder (e.g. OneDrive Desktop/Documents).</span>
          </div>
          <div style="display: flex; align-items: flex-start; gap: 8px; color: #fca5a5;">
            <span style="color: #ef4444; font-weight: bold;">✕</span>
            <span><strong>Do NOT</strong> choose the folder where this launcher itself is installed.</span>
          </div>
        </div>
      </div>

      <!-- 2. No Sunrise Needed -->
      <div class="help-card" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px; padding: 14px;">
        <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 13px; color: #a855f7; margin-bottom: 8px;">
          <span style="background: #a855f7; color: #0f172a; width: 20px; height: 20px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700;">2</span>
          You Do NOT Need Sunrise
        </div>
        <p style="font-size: 12.5px; color: #cbd5e1; line-height: 1.5; margin: 0;">
          You do <strong>not</strong> need to install Sunrise or find separate game files. This installer handles everything automatically: it downloads the correct Season of Arrivals build directly from Steam and deploys the Dawn mod into it.
        </p>
      </div>

      <!-- 3. Launching Troubles -->
      <div class="help-card" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px; padding: 14px;">
        <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 13px; color: #eab308; margin-bottom: 8px;">
          <span style="background: #eab308; color: #0f172a; width: 20px; height: 20px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700;">3</span>
          Game Not Launching?
        </div>
        <p style="font-size: 12.5px; color: #cbd5e1; line-height: 1.5; margin: 0 0 8px 0;">
          If clicking the <strong>Launch Game</strong> button does not open the game:
        </p>
        <div style="font-size: 12px; color: #e2e8f0; background: rgba(0,0,0,0.25); padding: 8px 12px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.06);">
          Click the small arrow next to the Launch button ➔ select <strong>"Open Game Folder"</strong>, and launch <code style="color: #fde047;">destiny2.exe</code> directly from File Explorer.
        </div>
      </div>

      <!-- 4. Reset & Clean Reinstall -->
      <div class="help-card" style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 10px; padding: 14px;">
        <div style="display: flex; align-items: center; gap: 8px; font-weight: 600; font-size: 13px; color: #22c55e; margin-bottom: 8px;">
          <span style="background: #22c55e; color: #0f172a; width: 20px; height: 20px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center; font-size: 11px; font-weight: 700;">4</span>
          Clean Reset (If Anything Breaks)
        </div>
        <p style="font-size: 12.5px; color: #cbd5e1; line-height: 1.5; margin: 0 0 8px 0;">
          If an installation gets interrupted or corrupted: click the <strong>Verify Files</strong> button next to Launch Game (or in the options menu) to scan and repair all game files via SteamDepot.
        </p>
        <p style="font-size: 12.5px; color: #94a3b8; line-height: 1.5; margin: 0;">
          For a total clean reset: open your game folder, <strong>delete all files inside it</strong>, and click <strong>Install</strong> to redownload from scratch.
        </p>
      </div>

    </div>

    <div class="modal-actions" style="margin-top: 14px; padding-top: 12px; border-top: 1px solid rgba(255,255,255,0.08); justify-content: flex-end;">
      <button class="modal-btn modal-btn-primary" type="button" onclick={onClose} style="min-width: 130px;">
        Got it!
      </button>
    </div>
  </div>
</div>

<style>
  .help-content-scroll::-webkit-scrollbar {
    width: 6px;
  }
  .help-content-scroll::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
  }
  .help-content-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }
  .help-content-scroll::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.25);
  }
</style>
