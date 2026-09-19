<script>
  let {
    isOpen = false,
    qrSvg = '',
    errorMessage = '',
    isLoading = false,
    initialTab = 'qr',
    savedUsername = '',
    onClose = () => {},
    onSubmitCredentials = () => {},
    onTabChange = () => {}
  } = $props();

  let activeTab = $state('qr'); // 'qr' | 'credentials'
  let username = $state('');
  let password = $state('');
  let localError = $state('');

  $effect(() => {
    if (initialTab) {
      activeTab = initialTab;
    }
  });

  $effect(() => {
    if (savedUsername && !username) {
      username = savedUsername;
    }
  });

  $effect(() => {
    localError = errorMessage;
  });

  function setTab(tab) {
    activeTab = tab;
    onTabChange(tab);
  }

  function handleClose() {
    localError = '';
    onClose();
  }

  function handleSubmit(e) {
    e.preventDefault();
    if (!username.trim() || isLoading) return;
    localError = '';
    if (username.includes('@')) {
      localError = 'Please enter your Steam Account Name (not your email address). Your Account Name is the login name you use for Steam.';
      return;
    }
    onSubmitCredentials({
      username: username.trim(),
      password: password.trim()
    });
  }
</script>

<div class="modal-overlay" class:hidden={!isOpen} id="steam-auth-modal">
  <div class="modal-card steam-auth-card">
    <div class="modal-header">
        <div class="modal-title-row">
          <svg class="steam-icon" viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M12 2C6.477 2 2 6.477 2 12c0 4.237 2.636 7.855 6.356 9.312l2.67-3.885a3.5 3.5 0 0 1-.026-.016l-3.328-1.37a2.5 2.5 0 1 1 .59-1.428l3.66 1.507a3.5 3.5 0 0 1 5.922-1.996 3.5 3.5 0 0 1 .156 4.95l-3.86 2.653A10.02 10.02 0 0 0 12 22c5.523 0 10-4.477 10-10S17.523 2 12 2z"/>
          </svg>
          <h3>Sign in to Steam</h3>
        </div>
        <button class="modal-close-btn" id="steam-modal-close" title="Cancel" onclick={handleClose}>&times;</button>
      </div>

      <!-- Auth Method Tabs -->
      <div class="auth-tabs">
        <button
          class="auth-tab-btn"
          class:active={activeTab === 'qr'}
          id="tab-btn-qr"
          type="button"
          disabled={isLoading}
          onclick={() => setTab('qr')}
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7"></rect>
            <rect x="14" y="3" width="7" height="7"></rect>
            <rect x="14" y="14" width="7" height="7"></rect>
            <rect x="3" y="14" width="7" height="7"></rect>
          </svg>
          <span>Steam Mobile QR</span>
        </button>
        <button
          class="auth-tab-btn"
          class:active={activeTab === 'credentials'}
          id="tab-btn-creds"
          type="button"
          disabled={isLoading}
          onclick={() => setTab('credentials')}
        >
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
          </svg>
          <span>Account Password</span>
        </button>
      </div>

      <!-- QR Code View -->
      <div class="auth-tab-content" class:hidden={activeTab !== 'qr'} id="auth-view-qr">
        <div class="qr-display-container">
          <div class="qr-code-box" id="steam-qr-box">
            {#if qrSvg}
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html qrSvg}
            {:else}
              <div class="qr-placeholder">
                <div class="qr-spinner"></div>
                <p>Generating Steam QR Code...</p>
              </div>
            {/if}
          </div>
          <div class="qr-instructions">
            <div class="qr-step">
              <span class="qr-step-num">1</span>
              <span>Open the <strong>Steam Mobile App</strong> on your phone</span>
            </div>
            <div class="qr-step">
              <span class="qr-step-num">2</span>
              <span>Tap the <strong>QR scanner icon</strong> in the top header</span>
            </div>
            <div class="qr-step">
              <span class="qr-step-num">3</span>
              <span>Point your camera at this QR code to sign in</span>
            </div>
          </div>
        </div>

        <div class="modal-actions">
          <button class="modal-btn modal-btn-cancel" id="steam-qr-cancel" type="button" onclick={handleClose}>
            Cancel
          </button>
          <button class="modal-btn modal-btn-secondary" id="switch-to-creds-btn" type="button" onclick={() => setTab('credentials')}>
            Use Password Instead
          </button>
        </div>
      </div>

      <!-- Credentials Form View -->
      <form class="auth-tab-content auth-form" class:hidden={activeTab !== 'credentials'} id="auth-view-creds" onsubmit={handleSubmit}>
          {#if localError}
            <div class="auth-error-banner" role="alert">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              <span>{localError}</span>
            </div>
          {/if}

          <div class="form-group">
            <label for="steam-username-input" class="form-label">Steam Account Name</label>
            <input
              type="text"
              id="steam-username-input"
              class="auth-input"
              placeholder="Account Name (not email or display name)"
              autocomplete="username"
              spellcheck="false"
              disabled={isLoading}
              bind:value={username}
            />
            <span class="input-hint">Use your Steam Account Name (the login ID you use for the Steam client), not your email address or display name.</span>
          </div>

          <div class="form-group">
            <label for="steam-password-input" class="form-label">Steam Password</label>
            <input
              type="password"
              id="steam-password-input"
              class="auth-input"
              placeholder="Enter your Steam password"
              autocomplete="current-password"
              disabled={isLoading}
              bind:value={password}
            />
          </div>

          <div class="auth-notice">
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            <span>Dawn does not store your login. Credentials are exchanged directly with Steam.</span>
          </div>

          <div class="modal-actions">
            <button class="modal-btn modal-btn-cancel" id="steam-modal-cancel" type="button" onclick={handleClose} disabled={isLoading}>
              Cancel
            </button>
            <button class="modal-btn modal-btn-primary" id="steam-modal-confirm" type="submit" disabled={isLoading || !username.trim()}>
              {#if isLoading}
                <span class="btn-spinner active" style="margin-right: 6px;"></span>
                <span>Authenticating...</span>
              {:else}
                <span>Sign In & Download</span>
              {/if}
            </button>
          </div>
        </form>
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

  .input-hint {
    display: block;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.5);
    margin-top: 4px;
    line-height: 1.3;
  }
</style>

