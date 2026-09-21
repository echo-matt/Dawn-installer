<script>
  import { onMount } from 'svelte';
  import { api } from './lib/api.js';

  import BackgroundShader from './components/BackgroundShader.svelte';
  import TitleBar from './components/TitleBar.svelte';
  import EmblemHero from './components/EmblemHero.svelte';
  import ProgressCard from './components/ProgressCard.svelte';
  import ActionControls from './components/ActionControls.svelte';
  import SteamAuthModal from './components/SteamAuthModal.svelte';
  import SteamGuardModal from './components/SteamGuardModal.svelte';
  import DebugConsole from './components/DebugConsole.svelte';
  import ConfirmUninstallModal from './components/ConfirmUninstallModal.svelte';
  import LinuxLaunchModal from './components/LinuxLaunchModal.svelte';
  import AppUpdateModal from './components/AppUpdateModal.svelte';
  import HelpModal from './components/HelpModal.svelte';

  // App State
  let isLinux = $state(false);
  let isLinuxLaunchModalOpen = $state(false);
  let isHelpModalOpen = $state(false);
  let isAppUpdateModalOpen = $state(false);
  let appUpdateInfo = $state(null);
  let isAppUpdating = $state(false);
  let appUpdateProgress = $state(null);
  let selectedFolder = $state('');
  let isUninstallModalOpen = $state(false);
  let selectedLanguage = $state('english');
  let spaceStatus = $state('dot-amber');
  let folderTitle = $state('Click to choose install directory');
  let hasGameInstalled = $state(false);
  let hasDawnInstalled = $state(false);
  let dawnVersion = $state(null);
  let latestDawnVersion = $state(null);
  let isOperationRunning = $state(false);
  let isVerifyMode = $state(false);

  // Progress Card State
  let progressVisible = $state(false);
  let progressTitle = $state('Ready');
  let progressStepDesc = $state('');
  let progressPercent = $state(0);
  let progressDetails = $state('Standby');

  // Modals & Auth State
  let isAuthModalOpen = $state(false);
  let isGuardModalOpen = $state(false);
  let isDebugConsoleOpen = $state(false);
  let activeAuthTab = $state('qr'); // 'qr' | 'credentials'
  let authErrorMessage = $state('');
  let guardErrorMessage = $state('');
  let isAuthSubmitting = $state(false);
  let savedSteamUsername = $state('');
  let qrSvg = $state('');
  let guardType = $state('code');
  let guardMessage = $state('Enter the code sent to your Steam Mobile App or email:');

  // Toast
  let toastMessage = $state('');
  let isToastVisible = $state(false);
  let toastTimer = null;

  function showToast(msg, duration = 3500) {
    toastMessage = msg;
    isToastVisible = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      isToastVisible = false;
    }, duration);
  }

  let isDawnUpdateAvailable = $derived.by(() => {
    if (!hasGameInstalled || !hasDawnInstalled || !latestDawnVersion) {
      return false;
    }
    if (!dawnVersion) {
      return true;
    }
    const cleanClient = dawnVersion.trim().toLowerCase().replace(/^v/, '');
    const cleanLatest = latestDawnVersion.trim().toLowerCase().replace(/^v/, '');
    if (cleanClient === cleanLatest) {
      return false;
    }
    // Handle hotfix and payload naming variants (e.g. 0.1.3-omega-fix and 0.1.3-hotfix)
    const clientBase = cleanClient.split('-')[0];
    const latestBase = cleanLatest.split('-')[0];
    if (clientBase && latestBase && clientBase === latestBase) {
      if (
        (cleanClient.includes('fix') && cleanLatest.includes('hotfix')) ||
        (cleanClient.includes('hotfix') && cleanLatest.includes('fix'))
      ) {
        return false;
      }
    }
    return true;
  });

  let installButtonText = $derived(
    isOperationRunning
      ? 'Working...'
      : !hasGameInstalled
        ? 'Install'
        : !hasDawnInstalled
          ? 'Install Dawn'
          : isDawnUpdateAvailable
            ? 'UPDATE'
            : 'Launch Game'
  );

  async function checkFolderStatus(folderPath) {
    if (!folderPath) {
      spaceStatus = 'dot-amber';
      hasGameInstalled = false;
      hasDawnInstalled = false;
      dawnVersion = null;
      folderTitle = 'Click to choose install directory';
      return;
    }

    const preflight = await api.validatePreflight(folderPath);
    if (!preflight.valid || preflight.error) {
      hasGameInstalled = false;
      hasDawnInstalled = false;
      spaceStatus = 'dot-red';
      folderTitle = `${folderPath} — ${preflight.error || 'Invalid destination folder'}`;
      showToast(preflight.error || 'Invalid destination folder selected.', 8000);
      return;
    }

    hasGameInstalled = Boolean(preflight.has_game);
    hasDawnInstalled = Boolean(preflight.has_dawn);
    if (preflight.dawn_version) {
      dawnVersion = preflight.dawn_version;
    } else if (!dawnVersion) {
      dawnVersion = null;
    }

    if (!latestDawnVersion) {
      api.getLatestDawnVersion().then((v) => {
        if (v) latestDawnVersion = v;
      }).catch(() => {});
    }

    if (preflight.has_game) {
      spaceStatus = 'dot-green';
      if (preflight.has_dawn) {
        if (isDawnUpdateAvailable) {
          folderTitle = `${folderPath} — Dawn mod update available (v${dawnVersion || '?'} -> v${latestDawnVersion})`;
        } else {
          folderTitle = `${folderPath} — Destiny 2 + Dawn ${dawnVersion ? 'v' + dawnVersion : ''} ready`;
        }
      } else {
        folderTitle = `${folderPath} — Destiny 2 installed (Dawn not deployed)`;
      }
    } else if (preflight.has_enough_space) {
      spaceStatus = 'dot-green';
      folderTitle = `${folderPath} — ${preflight.free_gb} GB free`;
    } else {
      spaceStatus = 'dot-red';
      folderTitle = `${folderPath} — Insufficient disk space (${preflight.free_gb} GB free, ~${preflight.required_gb} GB needed)`;
    }
  }

  async function handleSelectFolder() {
    try {
      const chosen = await api.selectGameFolder();
      if (chosen) {
        selectedFolder = chosen;
        localStorage.setItem('dawn_destiny2_path', chosen);
        await checkFolderStatus(chosen);
      }
    } catch (err) {
      showToast(`Folder selection error: ${err.message || err}`);
    }
  }

  function handleLanguageChange(lang) {
    selectedLanguage = lang;
    localStorage.setItem('dawn_selected_lang', lang);
  }

  async function launchGame() {
    if (isLinux) {
      isLinuxLaunchModalOpen = true;
      return;
    }
    showToast('Launching Destiny 2...');
    try {
      const res = await api.launchGame(selectedFolder, selectedLanguage);
      if (!res.success) {
        showToast(res.error || res.message || 'Failed to launch game', 7000);
      } else {
        showToast(res.message || 'Game launched successfully!', 3500);
      }
    } catch (err) {
      showToast(`Launch error: ${err.message || err}`, 7000);
    }
  }

  async function runDirectDawnInstall() {
    if (isOperationRunning) return;
    if (!selectedFolder) {
      showToast('Please select a destination folder first.');
      await handleSelectFolder();
      return;
    }

    isOperationRunning = true;
    progressVisible = true;
    progressPercent = 10;
    progressTitle = isDawnUpdateAvailable ? 'Updating Dawn Mod' : 'Installing Dawn Mod';
    progressStepDesc = 'Fetching & deploying latest Dawn release...';
    progressDetails = 'Checking GitHub releases...';

    try {
      const res = await api.installDawn(selectedFolder);
      if (res.success) {
        showToast(isDawnUpdateAvailable ? 'Dawn updated successfully!' : 'Dawn installed successfully!');
        if (latestDawnVersion) {
          dawnVersion = latestDawnVersion;
        }
        await checkFolderStatus(selectedFolder);
      } else {
        showToast(res.error || res.message || 'Dawn update failed');
      }
    } catch (err) {
      showToast(`Error: ${err.message || err}`);
    } finally {
      isOperationRunning = false;
      hideProgress();
    }
  }

  async function handleMainAction() {
    if (isOperationRunning) return;

    if (!selectedFolder) {
      showToast('Please select a destination folder first.');
      await handleSelectFolder();
      return;
    }

    const preflight = await api.validatePreflight(selectedFolder);
    if (!preflight.valid || preflight.error) {
      hasGameInstalled = false;
      hasDawnInstalled = false;
      spaceStatus = 'dot-red';
      const errMsg = preflight.error || 'Invalid destination folder selected.';
      folderTitle = `${selectedFolder} — ${errMsg}`;
      showToast(errMsg, 8000);
      return;
    }

    if (hasGameInstalled) {
      if (!hasDawnInstalled || isDawnUpdateAvailable) {
        await runDirectDawnInstall();
        return;
      }

      if (isLinux) {
        isLinuxLaunchModalOpen = true;
        return;
      }

      await launchGame();
      return;
    }

    // Fresh install flow: verify disk space before starting
    if (!preflight.has_game && !preflight.has_enough_space) {
      spaceStatus = 'dot-red';
      folderTitle = `${selectedFolder} — Insufficient disk space (${preflight.free_gb} GB free, ~${preflight.required_gb} GB needed)`;
      showToast(
        `Cannot install: Not enough hard drive space! You have ${preflight.free_gb} GB free, but Destiny 2 requires at least ~${preflight.required_gb} GB. Please choose a drive with more space.`,
        9000
      );
      return;
    }

    isVerifyMode = false;
    authErrorMessage = '';
    guardErrorMessage = '';
    isAuthSubmitting = false;
    activeAuthTab = 'qr';
    isAuthModalOpen = true;
    qrSvg = '';

    // Initiate background download in QR mode
    runDownloadFlow({ authMethod: 'qr' });
  }

  async function handleVerifyFiles() {
    if (isOperationRunning) return;

    if (!selectedFolder) {
      showToast('Please select a destination folder first.');
      await handleSelectFolder();
      return;
    }

    const preflight = await api.validatePreflight(selectedFolder);
    if (!preflight.valid || preflight.error) {
      hasGameInstalled = false;
      hasDawnInstalled = false;
      spaceStatus = 'dot-red';
      const errMsg = preflight.error || 'Invalid destination folder selected.';
      folderTitle = `${selectedFolder} — ${errMsg}`;
      showToast(errMsg, 8000);
      return;
    }

    isVerifyMode = true;
    authErrorMessage = '';
    guardErrorMessage = '';
    isAuthSubmitting = false;
    activeAuthTab = 'qr';
    isAuthModalOpen = true;
    qrSvg = '';

    runDownloadFlow({ authMethod: 'qr' });
  }

  async function handleAuthTabChange(newTab) {
    activeAuthTab = newTab;
    authErrorMessage = '';
    if (newTab === 'credentials') {
      // If QR download was running in background, stop it cleanly
      if (isOperationRunning) {
        try {
          await api.cancelDepotDownload();
        } catch (_) {}
        isOperationRunning = false;
        qrSvg = '';
        hideProgress();
      }
    } else if (newTab === 'qr') {
      if (!isOperationRunning) {
        qrSvg = '';
        runDownloadFlow({ authMethod: 'qr' });
      }
    }
  }

  async function runDownloadFlow({ authMethod = 'qr', steamUsername = null, steamPassword = null }) {
    isOperationRunning = true;
    progressVisible = true;
    progressPercent = 0;
    progressTitle = isVerifyMode ? 'Verifying Game Files' : 'Signing in to Steam';
    progressStepDesc = authMethod === 'qr'
      ? 'Scan the QR code with Steam Mobile'
      : 'Authenticating account credentials...';
    progressDetails = isVerifyMode ? 'Connecting to Steam for manifest validation...' : 'Connecting to Steam...';

    try {
      const res = await api.startDepotDownload({
        installRoot: selectedFolder,
        languageCode: selectedLanguage,
        authMethod,
        steamUsername,
        steamPassword,
        isVerify: isVerifyMode,
      });

      isAuthSubmitting = false;

      if (res.cancelled || !isOperationRunning) {
        if (!isAuthModalOpen) {
          hideProgress();
          showToast(isVerifyMode ? 'Verification cancelled.' : 'Download cancelled.');
        }
      } else if (res.success && isOperationRunning) {
        progressPercent = 100;
        progressTitle = isVerifyMode ? 'Verification Complete!' : 'Installation Complete!';
        progressStepDesc = isVerifyMode ? 'Game files verified and repaired' : 'Dawn and Destiny 2 are ready to play';
        progressDetails = 'Finished successfully';
        hasGameInstalled = true;
        hasDawnInstalled = true;
        isOperationRunning = false;
        isAuthModalOpen = false;
        isGuardModalOpen = false;
        showToast(isVerifyMode ? 'Files verified & repaired successfully!' : 'Dawn installed successfully!');
        api.log('INFO', `Installation flow finished successfully. Refreshing folder status for '${selectedFolder}'`);
        await checkFolderStatus(selectedFolder);
      } else {
        hideProgress();
        if (authErrorMessage) {
          isAuthModalOpen = true;
          activeAuthTab = 'credentials';
        } else if (res.error) {
          const errLower = res.error.toLowerCase();
          api.log('WARN', `Download flow error returned: ${res.error}`);
          if (
            errLower.includes('password') ||
            errLower.includes('steam guard') ||
            errLower.includes('rate limit') ||
            errLower.includes('twofactor') ||
            errLower.includes('timeout') ||
            errLower.includes('authenticate')
          ) {
            authErrorMessage = res.error;
            isAuthModalOpen = true;
            activeAuthTab = 'credentials';
          } else {
            showToast(`Error: ${res.error}`);
          }
        }
      }
    } catch (err) {
      isAuthSubmitting = false;
      hideProgress();
      api.log('ERROR', `Download flow exception: ${err.message || err}`);
      showToast(`Download error: ${err.message || err}`);
    } finally {
      isAuthSubmitting = false;
      if (hasGameInstalled) {
        isAuthModalOpen = false;
        isGuardModalOpen = false;
      }
      if (!hasGameInstalled && !isAuthModalOpen && !isGuardModalOpen) {
        hideProgress();
      }
    }
  }

  function hideProgress() {
    progressVisible = false;
    isOperationRunning = false;
    const card = document.getElementById('install-progress-card');
    if (card) card.classList.remove('active');
  }

  async function handleCancel() {
    api.log('INFO', 'User cancelled active operation');
    hideProgress();
    isAuthSubmitting = false;
    isAuthModalOpen = false;
    isGuardModalOpen = false;
    showToast('Operation cancelled.');
    try {
      await api.cancelDepotDownload();
    } catch (_) {}
  }

  function handleAuthClose() {
    isAuthSubmitting = false;
    isAuthModalOpen = false;
    if (isOperationRunning) {
      handleCancel();
    }
  }

  function handleAuthSubmitCredentials({ username, password }) {
    api.log('INFO', `Credentials submitted for user '${username}'`);
    savedSteamUsername = username;
    isAuthSubmitting = true;
    authErrorMessage = '';
    guardErrorMessage = '';
    isAuthModalOpen = true;
    runDownloadFlow({
      authMethod: 'credentials',
      steamUsername: username,
      steamPassword: password,
    });
  }

  async function handleGuardSubmit(code) {
    isGuardModalOpen = false;
    progressDetails = 'Submitting verification code to Steam...';
    await api.sendConsoleInput(code);
  }

  async function handleSwitchToQr() {
    isGuardModalOpen = false;
    try {
      await api.cancelDepotDownload();
    } catch (_) {}
    isOperationRunning = false;
    isAuthSubmitting = false;
    activeAuthTab = 'qr';
    qrSvg = '';
    authErrorMessage = '';
    guardErrorMessage = '';
    isAuthModalOpen = true;
    runDownloadFlow({ authMethod: 'qr' });
  }

  // Submenu Actions
  function handleRequestUninstall() {
    if (!selectedFolder) return showToast('Please select a game folder first');
    isUninstallModalOpen = true;
  }

  async function handleConfirmUninstall() {
    isUninstallModalOpen = false;
    if (!selectedFolder) return;

    showToast('Uninstalling Dawn mod...');
    isOperationRunning = true;
    progressVisible = true;
    progressTitle = 'Uninstalling Dawn';
    progressStepDesc = 'Restoring original Steam DLL and removing mod files...';
    progressPercent = 50;
    progressDetails = 'Restoring original steam_api64.dll and purging .dawn directory';

    try {
      const res = await api.uninstallDawn(selectedFolder);
      if (res.success) {
        showToast(res.message || 'Dawn uninstalled successfully');
        await checkFolderStatus(selectedFolder);
      } else {
        showToast(res.error || 'Failed to uninstall Dawn');
      }
    } catch (err) {
      showToast('Error during uninstall: ' + (err.message || err));
    } finally {
      isOperationRunning = false;
      hideProgress();
    }
  }

  async function handleRestoreBackup() {
    if (!selectedFolder) return showToast('Select folder first');
    showToast('Restoring backup...');
    progressVisible = true;
    progressTitle = 'Restoring Backup';
    progressPercent = 50;
    const res = await api.restoreDawn(selectedFolder);
    hideProgress();
    showToast(res.message || 'Restoration complete');
    await checkFolderStatus(selectedFolder);
  }

  async function handleClearCache() {
    if (!selectedFolder) return showToast('Select folder first');
    const res = await api.clearCache(selectedFolder);
    showToast(res.message || 'Cache cleared');
  }

  async function handleOpenFolder() {
    if (selectedFolder) {
      await api.openFolder(selectedFolder);
    }
  }

  async function handleOpenDocs() {
    await api.openDocs();
  }

  function handleCloseHelp() {
    isHelpModalOpen = false;
    localStorage.setItem('dawn_has_seen_help_guide', 'true');
    if (appUpdateInfo && appUpdateInfo.update_available) {
      isAppUpdateModalOpen = true;
    }
  }

  async function handleInstallAppUpdate() {
    if (!appUpdateInfo || !appUpdateInfo.asset_url || isAppUpdating) {
      if (appUpdateInfo?.html_url) {
        window.open(appUpdateInfo.html_url, '_blank');
      }
      return;
    }
    isAppUpdating = true;
    try {
      await api.installAppUpdate(appUpdateInfo.asset_url, appUpdateInfo.asset_name || 'DAWN-Setup.exe');
    } catch (err) {
      isAppUpdating = false;
      showToast(`Update error: ${err.message || err}`, 7000);
    }
  }

  onMount(async () => {
    // 0. Detect OS platform
    try {
      const plat = await api.getPlatform();
      isLinux = plat === 'linux';
    } catch (_) {}

    // 0.1 Fetch latest Dawn release version for update comparison
    try {
      const ver = await api.getLatestDawnVersion();
      if (ver) {
        latestDawnVersion = ver;
      }
    } catch (_) {}

    // 0.2 Check for Launcher self-updates
    api.checkAppUpdate().then((info) => {
      if (info && info.update_available) {
        appUpdateInfo = info;
        if (!isHelpModalOpen) {
          isAppUpdateModalOpen = true;
        }
      }
    }).catch(() => {});

    // 0.3 Automatically show Help & Guide on first launch
    const hasSeenGuide = localStorage.getItem('dawn_has_seen_help_guide');
    if (!hasSeenGuide) {
      isHelpModalOpen = true;
      localStorage.setItem('dawn_has_seen_help_guide', 'true');
    }

    // 1. Load saved preferences
    const savedPath = localStorage.getItem('dawn_destiny2_path');
    const savedLang = localStorage.getItem('dawn_selected_lang');

    if (savedLang) {
      selectedLanguage = savedLang;
    }

    if (savedPath) {
      selectedFolder = savedPath;
      await checkFolderStatus(savedPath);
    } else {
      const detected = await api.detectGameFolder();
      if (detected) {
        selectedFolder = detected;
        localStorage.setItem('dawn_destiny2_path', detected);
        await checkFolderStatus(detected);
      }
    }

    // 2. Setup event listeners
    const unlistenProgress = await api.onDepotProgress((data) => {
      if (!isOperationRunning) return;
      progressVisible = true;
      progressPercent = data.percent;
      progressTitle = isVerifyMode ? 'Verifying & Repairing Game Files' : 'Downloading Destiny 2 & Installing Dawn';
      progressStepDesc = `Account Authenticated • Language: ${selectedLanguage.toUpperCase()}`;
      progressDetails = data.status || `${data.percent}%`;

      isAuthSubmitting = false;
      if (isAuthModalOpen) {
        isAuthModalOpen = false;
      }
      if (isGuardModalOpen) {
        isGuardModalOpen = false;
      }
    });

    const unlistenQr = await api.onDepotQrCode((svg) => {
      if (svg && svg !== qrSvg) {
        qrSvg = svg;
      }
      if (activeAuthTab === 'qr') {
        isAuthModalOpen = true;
      }
    });

    const unlistenSteamGuard = await api.onDepotSteamGuard((data) => {
      guardType = data.type || 'code';
      guardMessage = data.message || 'Enter your Steam Guard verification code:';
      guardErrorMessage = '';
      isAuthSubmitting = false;
      isGuardModalOpen = true;
      isAuthModalOpen = false;
    });

    const unlistenAuthSuccess = await api.onAuthSuccess(() => {
      isAuthModalOpen = false;
      isGuardModalOpen = false;
      isAuthSubmitting = false;
      authErrorMessage = '';
      guardErrorMessage = '';
      if (isOperationRunning) {
        progressVisible = true;
        progressTitle = 'Signing in to Steam';
        progressStepDesc = `Account Authenticated • Language: ${selectedLanguage.toUpperCase()}`;
        progressDetails = 'Retrieving game manifests from Steam servers...';
      }
    });

    const unlistenAuthError = await api.onAuthError((payload) => {
      isAuthSubmitting = false;
      const msg = payload?.message || 'Steam authentication failed. Please check your credentials.';
      authErrorMessage = msg;
      guardErrorMessage = msg;
      activeAuthTab = 'credentials';
      isGuardModalOpen = false;
      isAuthModalOpen = true;
      showToast(msg);
    });

    const unlistenInstallerProg = await api.onInstallerProgress((data) => {
      if (!isOperationRunning) return;
      progressVisible = true;
      if (data.percent) progressPercent = data.percent;
      if (data.status) progressDetails = data.status;
    });

    const unlistenAppUpdateProg = await api.onAppUpdateProgress((data) => {
      appUpdateProgress = data;
    });

    return () => {
      if (typeof unlistenProgress === 'function') unlistenProgress();
      if (typeof unlistenQr === 'function') unlistenQr();
      if (typeof unlistenSteamGuard === 'function') unlistenSteamGuard();
      if (typeof unlistenAuthSuccess === 'function') unlistenAuthSuccess();
      if (typeof unlistenAuthError === 'function') unlistenAuthError();
      if (typeof unlistenInstallerProg === 'function') unlistenInstallerProg();
      if (typeof unlistenAppUpdateProg === 'function') unlistenAppUpdateProg();
    };
  });
</script>

<div class="app-window">
  <!-- Animated Fluid Background Canvas -->
  <BackgroundShader />

  <!-- UI Overlay Layer -->
  <div class="ui-layer">
    <!-- Title Bar -->
    <TitleBar dawnVersion={latestDawnVersion} onOpenDebugLogs={() => (isDebugConsoleOpen = true)} />

    <!-- Center Hero Logo -->
    <EmblemHero />

    <!-- Bottom Left Floating Progress Card -->
    <ProgressCard
      visible={progressVisible}
      title={progressTitle}
      stepDesc={progressStepDesc}
      percent={progressPercent}
      details={progressDetails}
      onCancel={handleCancel}
    />

    <!-- Bottom Right Action Center -->
    <ActionControls
      {selectedFolder}
      {spaceStatus}
      {folderTitle}
      {selectedLanguage}
      {installButtonText}
      {isOperationRunning}
      {hasGameInstalled}
      isUpdateAvailable={isDawnUpdateAvailable}
      {isLinux}
      onLinuxLaunchAttempt={() => (isLinuxLaunchModalOpen = true)}
      onSelectFolder={handleSelectFolder}
      onLanguageChange={handleLanguageChange}
      onInstall={handleMainAction}
      onVerifyFiles={handleVerifyFiles}
      onLaunchAnyway={launchGame}
      onUpdateDawn={runDirectDawnInstall}
      onUninstallDawn={handleRequestUninstall}
      onClearCache={handleClearCache}
      onOpenFolder={handleOpenFolder}
      onOpenDebugLogs={() => (isDebugConsoleOpen = true)}
      onOpenHelp={() => (isHelpModalOpen = true)}
    />

    <!-- Toast Notification -->
    <div class="toast-notification" class:show={isToastVisible} id="toast-notification">
      {toastMessage}
    </div>
  </div>

  <!-- Steam Auth Modal -->
  <SteamAuthModal
    isOpen={isAuthModalOpen}
    {qrSvg}
    errorMessage={authErrorMessage}
    isLoading={isAuthSubmitting}
    initialTab={activeAuthTab}
    savedUsername={savedSteamUsername}
    isVerify={isVerifyMode}
    onClose={handleAuthClose}
    onSubmitCredentials={handleAuthSubmitCredentials}
    onTabChange={handleAuthTabChange}
  />

  <!-- Steam Guard 2FA Modal -->
  <SteamGuardModal
    isOpen={isGuardModalOpen}
    {guardType}
    message={guardMessage}
    errorMessage={guardErrorMessage}
    onClose={handleCancel}
    onSubmitCode={handleGuardSubmit}
    onSwitchToQr={handleSwitchToQr}
  />

  <!-- Debug Console Modal -->
  <DebugConsole
    isOpen={isDebugConsoleOpen}
    onClose={() => (isDebugConsoleOpen = false)}
  />

  <!-- Confirm Uninstall Modal -->
  <ConfirmUninstallModal
    isOpen={isUninstallModalOpen}
    gameFolder={selectedFolder}
    onClose={() => (isUninstallModalOpen = false)}
    onConfirm={handleConfirmUninstall}
  />

  <!-- Linux Launch Unsupported Modal -->
  <LinuxLaunchModal
    isOpen={isLinuxLaunchModalOpen}
    onClose={() => (isLinuxLaunchModalOpen = false)}
  />

  <!-- Launcher Self-Update Modal -->
  <AppUpdateModal
    isOpen={isAppUpdateModalOpen}
    updateInfo={appUpdateInfo}
    isUpdating={isAppUpdating}
    updateProgress={appUpdateProgress}
    onClose={() => (isAppUpdateModalOpen = false)}
    onUpdate={handleInstallAppUpdate}
  />

  <!-- Installation Guide & Help Modal -->
  <HelpModal
    isOpen={isHelpModalOpen}
    onClose={handleCloseHelp}
  />
</div>
