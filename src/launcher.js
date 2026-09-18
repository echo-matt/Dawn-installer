import { BackgroundShader } from './background-shader.js';

class DawnInstallerApp {
  constructor() {
    this.bgCanvas = document.getElementById('bg-canvas');
    this.shader = null;

    // State
    this.gameRoot = localStorage.getItem('dawn_game_root') || '';
    this.selectedLanguage = localStorage.getItem('dawn_language') || 'english';
    this.preflightData = null;
    this.isOperationRunning = false;
    this.isDawnInstalled = false;

    this.init();
  }

  async init() {
    // 1. Initialize WebGL Shader Background
    if (this.bgCanvas) {
      this.shader = new BackgroundShader(this.bgCanvas);
    }

    // 2. Setup Window Controls
    this.setupWindowControls();

    // 3. Setup Context Bar & Language Controls
    this.setupContextControls();

    // 4. Setup Split Button & Popover Menu
    this.setupActionControls();

    // 5. Setup Steam Auth & Guard Modals
    this.setupSteamAuthModal();
    this.setupSteamGuardModal();

    // 6. Setup Operation Cancellation
    this.setupCancelControl();

    // 7. Setup IPC Event Listeners from Electron
    this.setupIPCListeners();

    // 7. Close menu on outside click
    document.addEventListener('click', (e) => {
      const installMenu = document.getElementById('install-menu');
      const arrowBtn = document.getElementById('install-arrow-btn');
      if (installMenu && arrowBtn && !arrowBtn.contains(e.target) && !installMenu.contains(e.target)) {
        installMenu.classList.remove('open');
        arrowBtn.classList.remove('active');
      }
    });

    // 8. Initial Preflight Check
    await this.initPreflight();
  }

  showToast(message, duration = 3000) {
    const toast = document.getElementById('toast-notification');
    if (!toast) return;
    toast.textContent = message;
    toast.classList.add('show');
    setTimeout(() => {
      toast.classList.remove('show');
    }, duration);
  }

  setupWindowControls() {
    const minBtn = document.getElementById('win-min');
    const closeBtn = document.getElementById('win-close');
    const isElectron = window.electronAPI && window.electronAPI.isElectron;

    if (minBtn) {
      minBtn.addEventListener('click', () => {
        if (isElectron) window.electronAPI.minimize();
      });
    }

    if (closeBtn) {
      closeBtn.addEventListener('click', () => {
        if (isElectron) window.electronAPI.close();
        else window.close();
      });
    }
  }

  setupContextControls() {
    // Language Dropdown
    const langSelect = document.getElementById('language-select');
    if (langSelect) {
      langSelect.value = this.selectedLanguage;
      langSelect.addEventListener('change', (e) => {
        this.selectedLanguage = e.target.value;
        localStorage.setItem('dawn_language', this.selectedLanguage);
        this.showToast(`Language set to ${langSelect.options[langSelect.selectedIndex].text}`);
      });
    }

    // Path selection triggers
    const trigger = document.getElementById('folder-action-btn') || document.getElementById('path-selector-trigger');
    const browseBtn = document.getElementById('opt-browse-folder') || document.getElementById('btn-browse-path');

    const handleBrowse = async (e) => {
      if (e) e.stopPropagation();
      if (this.isOperationRunning) return;

      if (window.electronAPI && window.electronAPI.selectGameFolder) {
        const selected = await window.electronAPI.selectGameFolder();
        if (selected) {
          await this.setAndValidatePath(selected);
        }
      } else {
        const simulated = prompt('Enter game directory path:', this.gameRoot || 'D:\\Games\\Destiny 2');
        if (simulated) {
          await this.setAndValidatePath(simulated);
        }
      }
    };

    if (trigger) trigger.addEventListener('click', handleBrowse);
    if (browseBtn) browseBtn.addEventListener('click', handleBrowse);
  }

  async initPreflight() {
    const isElectron = window.electronAPI && window.electronAPI.isElectron;

    if (this.gameRoot) {
      await this.setAndValidatePath(this.gameRoot);
      return;
    }

    if (isElectron && window.electronAPI.detectGameFolder) {
      try {
        const detected = await window.electronAPI.detectGameFolder();
        if (detected) {
          await this.setAndValidatePath(detected);
          return;
        }
      } catch (_) {}
    }

    this.updatePreflightUI({
      valid: false,
      path: 'Select folder...',
      freeGB: 0,
      hasEnoughSpace: false,
      writeable: false
    });
  }

  async setAndValidatePath(targetPath) {
    if (!targetPath) return;

    this.gameRoot = targetPath;
    localStorage.setItem('dawn_game_root', targetPath);

    const isElectron = window.electronAPI && window.electronAPI.isElectron;
    let data;

    if (isElectron && window.electronAPI.validatePreflight) {
      data = await window.electronAPI.validatePreflight(targetPath);
    } else {
      data = {
        valid: true,
        path: targetPath,
        exists: true,
        writeable: true,
        hasGame: targetPath.toLowerCase().includes('destiny'),
        freeGB: 167.5,
        requiredGB: 110,
        hasEnoughSpace: true
      };
    }

    this.preflightData = data;
    this.updatePreflightUI(data);

    // Check if Dawn is already installed
    if (isElectron && data.hasGame && window.electronAPI.validateGameFolder) {
      const check = await window.electronAPI.validateGameFolder(targetPath);
      this.isDawnInstalled = check.valid && check.hasPackages;
    }

    this.updateInstallButton();
  }

  updatePreflightUI(data) {
    const pathText = document.getElementById('path-display-text');
    const spaceDot = document.getElementById('space-dot');
    const spaceText = document.getElementById('space-text');
    const spaceItem = document.getElementById('space-item');

    if (pathText) {
      pathText.textContent = data.path || 'Select folder...';
      pathText.title = data.path || '';
    }

    if (spaceDot) {
      if (data.freeGB !== undefined && data.freeGB > 0) {
        if (spaceText) spaceText.textContent = `${data.freeGB} GB free`;
        spaceDot.className = 'badge-dot ' + (data.hasEnoughSpace ? 'dot-green' : 'dot-amber');
        spaceDot.title = data.hasEnoughSpace
          ? `${data.freeGB} GB available. Storage check passed.`
          : `${data.freeGB} GB available. Fresh install needs ~110 GB!`;
        if (spaceItem) {
          spaceItem.title = spaceDot.title;
        }
      } else {
        if (spaceText) spaceText.textContent = '110 GB required';
        spaceDot.className = 'badge-dot dot-amber';
        spaceDot.title = 'Destiny 2 requires approximately 110 GB of free space.';
        if (spaceItem) spaceItem.title = spaceDot.title;
      }
    }
  }

  updateInstallButton() {
    const btn = document.getElementById('install-main-btn');
    const text = document.getElementById('install-btn-text');
    const spinner = document.getElementById('btn-spinner');
    if (!btn || !text) return;

    if (this.isOperationRunning) {
      btn.disabled = true;
      if (spinner) spinner.classList.add('active');
      return;
    }

    btn.disabled = false;
    if (spinner) spinner.classList.remove('active');

    if (!this.gameRoot || !this.preflightData || !this.preflightData.valid) {
      text.textContent = 'Install';
      return;
    }

    if (this.preflightData.hasGame) {
      if (this.isDawnInstalled) {
        text.textContent = 'Launch';
      } else {
        text.textContent = 'Install';
      }
    } else {
      text.textContent = 'Install';
    }
  }

  setupActionControls() {
    const mainBtn = document.getElementById('install-main-btn');
    const arrowBtn = document.getElementById('install-arrow-btn');
    const menu = document.getElementById('install-menu');

    // Arrow Button Toggles Popover Menu Upwards
    if (arrowBtn && menu) {
      arrowBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        menu.classList.toggle('open');
        arrowBtn.classList.toggle('active');
      });
    }

    // Popover Menu Items
    const optBrowse = document.getElementById('opt-browse-folder');
    const optRestore = document.getElementById('opt-restore-backup');
    const optClear = document.getElementById('opt-clear-cache');
    const optOpenFolder = document.getElementById('opt-open-folder');
    const optOpenDocs = document.getElementById('opt-open-docs');

    if (optBrowse) {
      optBrowse.addEventListener('click', () => {
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');
        (document.getElementById('folder-action-btn') || document.getElementById('btn-browse-path'))?.click();
      });
    }

    if (optRestore) {
      optRestore.addEventListener('click', async () => {
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');
        await this.handleRestoreBackup();
      });
    }

    if (optClear) {
      optClear.addEventListener('click', async () => {
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');
        await this.handleClearCache();
      });
    }

    if (optOpenFolder) {
      optOpenFolder.addEventListener('click', () => {
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');
        if (window.electronAPI && this.gameRoot) {
          window.electronAPI.openFolder(this.gameRoot);
        }
      });
    }

    if (optOpenDocs) {
      optOpenDocs.addEventListener('click', () => {
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');
        if (window.electronAPI) {
          window.electronAPI.openDocs();
        }
      });
    }

    // Main Install / Launch Button
    if (mainBtn) {
      mainBtn.addEventListener('click', async () => {
        if (this.isOperationRunning) return;

        if (!this.gameRoot || !this.preflightData || !this.preflightData.valid) {
          (document.getElementById('folder-action-btn') || document.getElementById('btn-browse-path'))?.click();
          return;
        }

        if (this.preflightData.hasGame) {
          if (this.isDawnInstalled) {
            await this.launchGame();
          } else {
            await this.runDirectInstall();
          }
        } else {
          this.promptSteamLogin();
        }
      });
    }
  }

  setupCancelControl() {
    const cancelBtn = document.getElementById('cancel-operation-btn');
    if (cancelBtn) {
      cancelBtn.addEventListener('click', async () => {
        this.isOperationRunning = false;
        this._isQrPending = false;
        this.hideProgressCard();
        this.updateInstallButton();
        if (window.electronAPI && window.electronAPI.cancelDepotDownload) {
          await window.electronAPI.cancelDepotDownload();
        }
        this.showToast('Download cancelled');
      });
    }
  }

  setupIPCListeners() {
    if (!window.electronAPI) return;

    window.electronAPI.onDepotOutput((text) => {
      // Output logging available if needed
    });

    window.electronAPI.onSteamQrCode((svg) => {
      const qrBox = document.getElementById('steam-qr-box');
      if (qrBox) {
        qrBox.innerHTML = svg;
      }
    });

    window.electronAPI.onSteamGuardNeeded((data) => {
      if (data.type === 'mobile_confirm') {
        this.updateProgress(0, '📱 Please approve the login prompt in your Steam Mobile App...');
        this.showToast('Check your phone: approve login in Steam Mobile App', 6000);
      } else {
        const guardModal = document.getElementById('steam-guard-modal');
        const guardDesc = document.getElementById('steam-guard-desc');
        const guardInput = document.getElementById('steam-guard-code-input');
        if (guardDesc && data.message) guardDesc.textContent = data.message;
        if (guardInput) {
          guardInput.value = '';
          setTimeout(() => guardInput.focus(), 60);
        }
        guardModal?.classList.remove('hidden');
      }
    });

    window.electronAPI.onDepotProgress(({ percent, status }) => {
      const authModal = document.getElementById('steam-auth-modal');
      if (authModal && !authModal.classList.contains('hidden')) {
        authModal.classList.add('hidden');
        this._isQrPending = false;
      }
      const guardModal = document.getElementById('steam-guard-modal');
      if (guardModal && !guardModal.classList.contains('hidden')) {
        guardModal.classList.add('hidden');
      }
      this.updateProgress(percent, status);
    });

    window.electronAPI.onProgress(({ percent, status }) => {
      this.updateProgress(percent, status);
    });

    window.electronAPI.onLog(({ message, type }) => {
      // Log handling
    });
  }

  appendConsoleLog(text) {
    // Retained for compatibility
  }

  showProgressCard() {
    const progressCard = document.getElementById('install-progress-card');
    if (progressCard) progressCard.classList.add('active');
  }

  hideProgressCard() {
    const progressCard = document.getElementById('install-progress-card');
    if (progressCard) progressCard.classList.remove('active');
  }

  updateProgress(percent, statusText) {
    const progressCard = document.getElementById('install-progress-card');
    const fill = document.getElementById('progress-bar-fill');
    const pctText = document.getElementById('progress-percent');
    const statusElem = document.getElementById('progress-details');

    if (this.isOperationRunning && progressCard) {
      progressCard.classList.add('active');
    }
    if (fill) fill.style.width = `${percent}%`;
    if (pctText) pctText.textContent = `${percent}%`;
    if (statusElem && statusText) statusElem.textContent = statusText;
  }

  async runDepotDownloadAndInstall({ authMethod = 'qr', steamUsername = '', steamPassword = '' }) {
    this.isOperationRunning = true;
    this.updateInstallButton();

    const progressCard = document.getElementById('install-progress-card');
    const title = document.getElementById('progress-title');
    const stepDesc = document.getElementById('progress-step-desc');

    if (progressCard) progressCard.classList.add('active');
    if (title) title.textContent = 'Downloading Destiny 2 & Installing Dawn';
    if (stepDesc) {
      stepDesc.textContent = authMethod === 'qr'
        ? `Authentication: Steam Mobile QR • Depots: 1085661 (Base) + ${this.selectedLanguage}`
        : `Account: ${steamUsername} • Depots: 1085661 (Base) + ${this.selectedLanguage}`;
    }

    this.updateProgress(0, authMethod === 'qr' ? 'Waiting for Steam Mobile App scan...' : 'Connecting to Steam...');

    try {
      if (window.electronAPI && window.electronAPI.startDepotDownload) {
        const result = await window.electronAPI.startDepotDownload({
          installRoot: this.gameRoot,
          languageCode: this.selectedLanguage,
          authMethod,
          steamUsername,
          steamPassword
        });

        if (result.success) {
          this.isDawnInstalled = true;
          this.showToast('Installation completed successfully!');
          this.updateProgress(100, 'Dawn is ready to play.');
        } else if (result.cancelled) {
          this.hideProgressCard();
          this.showToast('Download cancelled');
        } else {
          this.showToast(result.error || 'Download failed');
          this.updateProgress(0, result.error || 'Download failed');
        }
      }
    } catch (err) {
      this.showToast(`Error: ${err.message}`);
      this.updateProgress(0, `Error: ${err.message}`);
    } finally {
      this.isOperationRunning = false;
      this._isQrPending = false;
      this.updateInstallButton();
    }
  }

  async runDirectInstall() {
    this.isOperationRunning = true;
    this.updateInstallButton();

    const progressCard = document.getElementById('install-progress-card');
    const title = document.getElementById('progress-title');
    const stepDesc = document.getElementById('progress-step-desc');

    if (progressCard) progressCard.classList.add('active');
    if (title) title.textContent = 'Deploying Dawn Mod';
    if (stepDesc) stepDesc.textContent = 'Injecting hooks, scripts, and initial configurations';

    this.updateProgress(10, 'Running Dawn installer engine...');

    try {
      if (window.electronAPI && window.electronAPI.installDawn) {
        const result = await window.electronAPI.installDawn(this.gameRoot);
        if (result.success) {
          this.isDawnInstalled = true;
          this.showToast('Dawn installed successfully!');
          this.updateProgress(100, 'Ready to launch.');
        } else {
          this.showToast(result.message || 'Installation failed');
        }
      }
    } catch (err) {
      this.showToast(`Error: ${err.message}`);
    } finally {
      this.isOperationRunning = false;
      this.updateInstallButton();
    }
  }

  async launchGame() {
    if (!window.electronAPI) return;
    this.showToast('Launching Dawn...');
    const result = await window.electronAPI.launchGame(this.gameRoot);
    if (!result.success) {
      this.showToast(result.message || 'Failed to launch game');
    }
  }

  async handleRestoreBackup() {
    if (!window.electronAPI || !this.gameRoot) {
      this.showToast('No game folder selected');
      return;
    }
    this.showToast('Restoring previous Dawn backup...');
    const result = await window.electronAPI.restoreDawn(this.gameRoot);
    this.showToast(result.message);
  }

  async handleClearCache() {
    if (!window.electronAPI || !this.gameRoot) {
      this.showToast('No game folder selected');
      return;
    }
    const result = await window.electronAPI.clearCache(this.gameRoot);
    this.showToast(result.message);
  }

  setupSteamAuthModal() {
    const modal = document.getElementById('steam-auth-modal');
    const tabBtnQr = document.getElementById('tab-btn-qr');
    const tabBtnCreds = document.getElementById('tab-btn-creds');
    const viewQr = document.getElementById('auth-view-qr');
    const viewCreds = document.getElementById('auth-view-creds');
    const switchToCredsBtn = document.getElementById('switch-to-creds-btn');
    const credsForm = document.getElementById('auth-view-creds');
    const userInput = document.getElementById('steam-username-input');
    const passInput = document.getElementById('steam-password-input');
    const closeBtn = document.getElementById('steam-modal-close');
    const cancelBtnQr = document.getElementById('steam-qr-cancel');
    const cancelBtnCreds = document.getElementById('steam-modal-cancel');

    this.activeAuthTab = 'qr';

    const switchTab = (tab) => {
      this.activeAuthTab = tab;
      if (tab === 'qr') {
        tabBtnQr?.classList.add('active');
        tabBtnCreds?.classList.remove('active');
        viewQr?.classList.remove('hidden');
        viewCreds?.classList.add('hidden');
        if (!this.isOperationRunning && modal && !modal.classList.contains('hidden')) {
          this.startQrLoginSession();
        }
      } else {
        tabBtnCreds?.classList.add('active');
        tabBtnQr?.classList.remove('active');
        viewCreds?.classList.remove('hidden');
        viewQr?.classList.add('hidden');
        if (this.isOperationRunning && this._isQrPending) {
          if (window.electronAPI && window.electronAPI.cancelDepotDownload) {
            window.electronAPI.cancelDepotDownload();
          }
          this.isOperationRunning = false;
          this._isQrPending = false;
        }
        setTimeout(() => userInput?.focus(), 60);
      }
    };

    if (tabBtnQr) tabBtnQr.addEventListener('click', () => switchTab('qr'));
    if (tabBtnCreds) tabBtnCreds.addEventListener('click', () => switchTab('creds'));
    if (switchToCredsBtn) switchToCredsBtn.addEventListener('click', () => switchTab('creds'));

    const closeModal = () => {
      modal?.classList.add('hidden');
      if (this._isQrPending) {
        if (window.electronAPI && window.electronAPI.cancelDepotDownload) {
          window.electronAPI.cancelDepotDownload();
        }
        this.isOperationRunning = false;
        this._isQrPending = false;
        this.hideProgressCard();
        this.updateInstallButton();
      }
    };

    if (closeBtn) closeBtn.addEventListener('click', closeModal);
    if (cancelBtnQr) cancelBtnQr.addEventListener('click', closeModal);
    if (cancelBtnCreds) cancelBtnCreds.addEventListener('click', closeModal);

    if (credsForm) {
      credsForm.addEventListener('submit', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const u = userInput ? userInput.value.trim() : '';
        const p = passInput ? passInput.value : '';

        if (!u) {
          userInput?.focus();
          this.showToast('Please enter your Steam username.');
          return;
        }
        if (!p) {
          passInput?.focus();
          this.showToast('Please enter your Steam password.');
          return;
        }

        modal?.classList.add('hidden');
        this.runDepotDownloadAndInstall({
          authMethod: 'credentials',
          steamUsername: u,
          steamPassword: p
        });
      });
    }
  }

  startQrLoginSession() {
    this._isQrPending = true;
    const qrBox = document.getElementById('steam-qr-box');
    if (qrBox) {
      qrBox.innerHTML = `
        <div class="qr-placeholder">
          <div class="qr-spinner"></div>
          <p>Connecting to Steam...<br><span style="font-size:10.5px;color:#64748b;">Generating QR Code</span></p>
        </div>
      `;
    }

    this.runDepotDownloadAndInstall({
      authMethod: 'qr',
      steamUsername: '',
      steamPassword: ''
    });
  }

  promptSteamLogin() {
    const modal = document.getElementById('steam-auth-modal');
    if (!modal) return;

    modal.classList.remove('hidden');

    const tabBtnQr = document.getElementById('tab-btn-qr');
    const tabBtnCreds = document.getElementById('tab-btn-creds');
    const viewQr = document.getElementById('auth-view-qr');
    const viewCreds = document.getElementById('auth-view-creds');
    const userInput = document.getElementById('steam-username-input');
    const passInput = document.getElementById('steam-password-input');

    if (userInput) userInput.value = '';
    if (passInput) passInput.value = '';

    // Default to QR tab
    this.activeAuthTab = 'qr';
    tabBtnQr?.classList.add('active');
    tabBtnCreds?.classList.remove('active');
    viewQr?.classList.remove('hidden');
    viewCreds?.classList.add('hidden');

    this.startQrLoginSession();
  }

  setupSteamGuardModal() {
    const guardModal = document.getElementById('steam-guard-modal');
    const guardForm = document.getElementById('steam-guard-form');
    const guardInput = document.getElementById('steam-guard-code-input');
    const closeBtn = document.getElementById('steam-guard-close');
    const cancelBtn = document.getElementById('steam-guard-cancel');

    const closeHandler = (e) => {
      if (e) {
        e.preventDefault();
        e.stopPropagation();
      }
      guardModal?.classList.add('hidden');
      this.isOperationRunning = false;
      this._isQrPending = false;
      this.hideProgressCard();
      this.updateInstallButton();
      if (window.electronAPI && window.electronAPI.cancelDepotDownload) {
        window.electronAPI.cancelDepotDownload();
      }
    };

    if (closeBtn) closeBtn.addEventListener('click', closeHandler);
    if (cancelBtn) cancelBtn.addEventListener('click', closeHandler);
    if (guardModal) {
      guardModal.addEventListener('click', (e) => {
        if (e.target === guardModal) closeHandler(e);
      });
    }

    if (guardForm) {
      guardForm.addEventListener('submit', (e) => {
        e.preventDefault();
        e.stopPropagation();
        const code = guardInput ? guardInput.value.trim() : '';
        if (!code) {
          guardInput?.focus();
          this.showToast('Please enter the Steam Guard code.');
          return;
        }

        guardModal?.classList.add('hidden');
        if (window.electronAPI && window.electronAPI.sendConsoleInput) {
          window.electronAPI.sendConsoleInput(code);
        }
      });
    }
  }
}

window.addEventListener('DOMContentLoaded', () => {
  new DawnInstallerApp();
});
