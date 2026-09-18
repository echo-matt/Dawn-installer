<script>
  let {
    isOpen = false,
    guardType = 'code',
    message = 'Enter the Steam Guard code sent to your email or authenticator:',
    errorMessage = '',
    onClose = () => {},
    onSubmitCode = () => {}
  } = $props();

  let codeInput = $state('');

  function handleSubmit(e) {
    e.preventDefault();
    if (!codeInput.trim()) return;
    onSubmitCode(codeInput.trim());
    codeInput = '';
  }
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="steam-guard-modal">
  <div class="modal-card">
      <div class="modal-header">
        <div class="modal-title-row">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="#60a5fa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
          <h3>Steam Guard Verification</h3>
        </div>
        <button class="modal-close-btn" id="steam-guard-close" title="Cancel" onclick={onClose}>&times;</button>
      </div>

      {#if errorMessage}
        <div class="auth-error-banner" role="alert">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span>{errorMessage}</span>
        </div>
      {/if}

      <p class="modal-desc" id="steam-guard-desc">
        {message}
      </p>

      {#if guardType === 'code'}
        <form class="auth-form" id="steam-guard-form" onsubmit={handleSubmit}>
          <div class="form-group">
            <input
              type="text"
              id="steam-guard-code-input"
              class="auth-input guard-input"
              placeholder="e.g. W8X4R"
              maxlength="10"
              autocomplete="one-time-code"
              spellcheck="false"
              bind:value={codeInput}
            />
          </div>

          <div class="modal-actions">
            <button class="modal-btn modal-btn-cancel" id="steam-guard-cancel" type="button" onclick={onClose}>
              Cancel
            </button>
            <button class="modal-btn modal-btn-primary" id="steam-guard-confirm" type="submit">
              Submit Code
            </button>
          </div>
        </form>
      {:else}
        <div class="qr-placeholder" style="margin-top: 20px;">
          <div class="qr-spinner"></div>
          <p>Waiting for Mobile Confirmation...</p>
        </div>
        <div class="modal-actions" style="margin-top: 20px;">
          <button class="modal-btn modal-btn-cancel" type="button" onclick={onClose}>
            Cancel
          </button>
        </div>
      {/if}
    </div>
  </div>

<style>
  .auth-error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #fca5a5;
    padding: 10px 12px;
    border-radius: 8px;
    font-size: 12px;
    line-height: 1.4;
    margin-bottom: 12px;
    animation: fadeIn 0.2s ease-out;
  }

  .auth-error-banner svg {
    flex-shrink: 0;
    color: #ef4444;
  }
</style>

