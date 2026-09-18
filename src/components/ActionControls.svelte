<script>
  let {
    selectedFolder = '',
    spaceStatus = 'dot-amber',
    folderTitle = 'Click to choose install directory',
    selectedLanguage = 'english',
    installButtonText = 'Install',
    isOperationRunning = false,
    isLinux = false,
    onSelectFolder = () => {},
    onLanguageChange = () => {},
    onInstall = () => {},
    onRestoreBackup = () => {},
    onClearCache = () => {},
    onOpenFolder = () => {},
    onOpenDocs = () => {},
    onOpenDebugLogs = () => {},
    onUpdateDawn = () => {},
    onUninstallDawn = () => {},
    isUpdateAvailable = false,
    onLaunchAnyway = () => {}
  } = $props();

  let isMenuOpen = $state(false);

  function toggleMenu() {
    if (isOperationRunning) return;
    isMenuOpen = !isMenuOpen;
  }

  function closeMenu() {
    isMenuOpen = false;
  }

  function handleFolderClick() {
    if (isOperationRunning) return;
    closeMenu();
    onSelectFolder();
  }

  function handleMainAction() {
    if (isOperationRunning) return;
    closeMenu();
    onInstall();
  }

  function formatPath(p) {
    if (!p) return 'Select folder...';
    if (p.length > 25) {
      return p.slice(0, 10) + '...' + p.slice(-12);
    }
    return p;
  }
</script>

<svelte:window onclick={(e) => {
  const target = e.target;
  if (!target.closest('.button-wrapper')) {
    closeMenu();
  }
}} />

<footer class="bottom-right-cluster">
  <div class="action-row">
    <!-- 1. Deep Blue Transparent Select Folder Button -->
    <button
      class="deep-blue-btn folder-btn"
      id="folder-action-btn"
      title={folderTitle}
      disabled={isOperationRunning}
      onclick={handleFolderClick}
    >
      <svg class="folder-icon" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      <span class="folder-btn-text" id="path-display-text">{formatPath(selectedFolder)}</span>
      <span class="badge-dot {spaceStatus}" id="space-dot"></span>
    </button>

    <!-- 2. Deep Blue Transparent Language Selector Button -->
    <div class="deep-blue-btn language-btn-wrapper" title="Select game language depot">
      <svg class="globe-icon" viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="2" y1="12" x2="22" y2="12"></line>
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
      </svg>
      <select
        id="language-select"
        class="language-select-input"
        value={selectedLanguage}
        disabled={isOperationRunning}
        onchange={(e) => onLanguageChange(e.currentTarget.value)}
      >
        <option value="english">EN</option>
        <option value="french">FR</option>
        <option value="german">DE</option>
        <option value="italian">IT</option>
        <option value="japanese">JA</option>
        <option value="brazilian">PT-BR</option>
        <option value="spanish">ES</option>
        <option value="russian">RU</option>
        <option value="polish">PL</option>
        <option value="schinese">ZH-CN</option>
        <option value="tchinese">ZH-TW</option>
        <option value="latam">ES-LA</option>
        <option value="koreana">KO</option>
      </select>
    </div>

    <!-- 3. Crisp White Split Install Button & Options Menu -->
    <div class="button-wrapper">
      <div class="split-button" class:is-update={isUpdateAvailable} class:disabled={isOperationRunning} id="split-button-container">
        <button
          class="btn-main"
          id="install-main-btn"
          disabled={isOperationRunning}
          onclick={handleMainAction}
        >
          {#if isOperationRunning}
            <span class="btn-spinner active" id="btn-spinner"></span>
          {:else if isUpdateAvailable}
            <svg class="update-icon" viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
          {/if}
          <span id="install-btn-text">{installButtonText}</span>
        </button>
        <div class="btn-divider"></div>
        <button
          class="btn-arrow"
          id="install-arrow-btn"
          title="Installer Options"
          disabled={isOperationRunning}
          onclick={toggleMenu}
        >
          <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="18 15 12 9 6 15"></polyline>
          </svg>
        </button>
      </div>

      <!-- Popover Menu -->
      <div class="popover-menu" class:active={isMenuOpen} class:open={isMenuOpen} id="install-menu">
        {#if isUpdateAvailable && !isLinux}
          <button type="button" class="menu-item" id="opt-launch-anyway" onclick={() => { closeMenu(); onLaunchAnyway(); }}>
            <span>Launch Game (Skip Update)</span>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
          </button>
          <div class="menu-divider"></div>
        {/if}
        <button type="button" class="menu-item" id="opt-browse-folder" onclick={() => { closeMenu(); onSelectFolder(); }}>
          <span>Change Game Directory...</span>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
        </button>
        <button type="button" class="menu-item" id="opt-update-dawn" onclick={() => { closeMenu(); onUpdateDawn(); }}>
          <span>Update / Reinstall Dawn Mod</span>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
        </button>
        <button type="button" class="menu-item" id="opt-clear-cache" onclick={() => { closeMenu(); onClearCache(); }}>
          <span>Clear Dawn Cache</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="menu-item" id="opt-open-folder" onclick={() => { closeMenu(); onOpenFolder(); }}>
          <span>Open Game Folder</span>
        </button>
        <button type="button" class="menu-item" id="opt-open-logs" onclick={() => { closeMenu(); onOpenDebugLogs(); }}>
          <span>View Debug Logs</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="menu-item menu-item-danger" id="opt-uninstall-dawn" onclick={() => { closeMenu(); onUninstallDawn(); }}>
          <span>Uninstall Dawn Mod</span>
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            <line x1="10" y1="11" x2="10" y2="17"></line>
            <line x1="14" y1="11" x2="14" y2="17"></line>
          </svg>
        </button>
      </div>
    </div>
  </div>
</footer>
