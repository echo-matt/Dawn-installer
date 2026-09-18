const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');

ipcMain.handle('detect-game-folder', async () => null);

app.whenReady().then(async () => {
  const win = new BrowserWindow({
    width: 1040,
    height: 585,
    show: false,
    frame: false,
    transparent: true,
    webPreferences: {
      offscreen: true,
      preload: path.join(__dirname, 'electron', 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  win.webContents.on('console-message', (e, level, message, line, sourceId) => {
    console.log(`[Renderer ${level}]`, message);
  });

  const brainDir = 'C:/Users/Matteo/.gemini/antigravity/brain/dcd6b415-40d1-4e48-9211-ded1dcbcdaf9';

  await win.loadFile(path.join(__dirname, 'dist', 'index.html'));
  await new Promise(r => setTimeout(r, 1400));

  // 1. Initial State
  let image = await win.capturePage();
  fs.writeFileSync(path.join(brainDir, 'dawn_standalone_installer_preview.png'), image.toPNG());

  // 2. Folder selected state
  try {
    await win.webContents.executeJavaScript(`
      (() => {
        const pathText = document.getElementById('path-display-text');
        const spaceDot = document.getElementById('space-dot');
        if (pathText) pathText.textContent = "D:\\\\Games\\\\Destiny 2";
        if (spaceDot) {
          spaceDot.className = 'badge-dot dot-green';
          spaceDot.title = '184.2 GB free';
        }
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_folder_selected_preview.png'), image.toPNG());
  } catch (err) {
    console.error('Step 2 error:', err);
  }

  // 3. Popover Menu open state
  try {
    await win.webContents.executeJavaScript(`
      (() => {
        const menu = document.getElementById('install-menu');
        const arrowBtn = document.getElementById('install-arrow-btn');
        if (menu) menu.classList.add('open');
        if (arrowBtn) arrowBtn.classList.add('active');
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_standalone_installer_menu_preview.png'), image.toPNG());
  } catch (err) {
    console.error('Step 3 error:', err);
  }

  // 3b. Steam Authentication Modal (In-App QR Code View)
  try {
    const testSvgPath = path.join(brainDir, 'scratch', 'test_qr.svg');
    const svgContent = fs.existsSync(testSvgPath) ? fs.readFileSync(testSvgPath, 'utf8') : '';

    await win.webContents.executeJavaScript(`
      (() => {
        const menu = document.getElementById('install-menu');
        const arrowBtn = document.getElementById('install-arrow-btn');
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');

        const modal = document.getElementById('steam-auth-modal');
        if (modal) modal.classList.remove('hidden');

        const tabBtnQr = document.getElementById('tab-btn-qr');
        const tabBtnCreds = document.getElementById('tab-btn-creds');
        const viewQr = document.getElementById('auth-view-qr');
        const viewCreds = document.getElementById('auth-view-creds');

        if (tabBtnQr) tabBtnQr.className = 'auth-tab-btn active';
        if (tabBtnCreds) tabBtnCreds.className = 'auth-tab-btn';
        if (viewQr) viewQr.className = 'auth-tab-content';
        if (viewCreds) viewCreds.className = 'auth-tab-content hidden auth-form';

        const qrBox = document.getElementById('steam-qr-box');
        if (qrBox && ${JSON.stringify(svgContent)}) {
          qrBox.innerHTML = ${JSON.stringify(svgContent)};
        }
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_steam_qr_modal_preview.png'), image.toPNG());

    // 3c. Switch to Credentials View
    await win.webContents.executeJavaScript(`
      (() => {
        const tabBtnQr = document.getElementById('tab-btn-qr');
        const tabBtnCreds = document.getElementById('tab-btn-creds');
        const viewQr = document.getElementById('auth-view-qr');
        const viewCreds = document.getElementById('auth-view-creds');

        if (tabBtnCreds) tabBtnCreds.className = 'auth-tab-btn active';
        if (tabBtnQr) tabBtnQr.className = 'auth-tab-btn';
        if (viewCreds) viewCreds.className = 'auth-tab-content auth-form';
        if (viewQr) viewQr.className = 'auth-tab-content hidden';

        const userInput = document.getElementById('steam-username-input');
        if (userInput) userInput.value = 'GuardianPlayer';
        const passInput = document.getElementById('steam-password-input');
        if (passInput) passInput.value = 'SecretSteamPassword123!';
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_steam_login_credentials_preview.png'), image.toPNG());

    // 3d. Steam Guard Modal (In-App 2FA / Steam Guard)
    await win.webContents.executeJavaScript(`
      (() => {
        const modal = document.getElementById('steam-auth-modal');
        if (modal) modal.classList.add('hidden');

        const guardModal = document.getElementById('steam-guard-modal');
        if (guardModal) guardModal.classList.remove('hidden');
        const guardInput = document.getElementById('steam-guard-code-input');
        if (guardInput) guardInput.value = 'W8X4R';
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_steam_guard_modal_preview.png'), image.toPNG());

    // Close guard modal
    await win.webContents.executeJavaScript(`
      (() => {
        const guardModal = document.getElementById('steam-guard-modal');
        if (guardModal) guardModal.classList.add('hidden');
      })()
    `);
  } catch (err) {
    console.error('Step 3 error:', err);
  }

  // 4. Progress active state (NO CONSOLE, clean glowing in-app progress)
  try {
    await win.webContents.executeJavaScript(`
      (() => {
        const menu = document.getElementById('install-menu');
        const arrowBtn = document.getElementById('install-arrow-btn');
        if (menu) menu.classList.remove('open');
        if (arrowBtn) arrowBtn.classList.remove('active');

        const progressCard = document.getElementById('install-progress-card');
        const progressBar = document.getElementById('progress-bar-fill');
        const progressPercent = document.getElementById('progress-percent');
        const progressTitle = document.getElementById('progress-title');
        const progressStepDesc = document.getElementById('progress-step-desc');
        const progressDetails = document.getElementById('progress-details');
        const mainBtn = document.getElementById('install-main-btn');
        const btnSpinner = document.getElementById('btn-spinner');

        if (progressCard) progressCard.classList.add('active');
        if (progressBar) progressBar.style.width = '64%';
        if (progressPercent) progressPercent.textContent = '64%';
        if (progressTitle) progressTitle.textContent = 'Downloading Destiny 2 & Installing Dawn';
        if (progressStepDesc) progressStepDesc.textContent = 'Account: GuardianPlayer • Depots: 1085661 (Base) + English';
        if (progressDetails) progressDetails.textContent = '48.2 GB / 75.3 GB (34.8 MB/s) — Base Depot 64%';
        if (mainBtn) mainBtn.disabled = true;
        if (btnSpinner) btnSpinner.classList.add('active');
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_standalone_installer_downloading_preview.png'), image.toPNG());
  } catch (err) {
    console.error('Step 4 error:', err);
  }

  // 5. Test clicking cancel button to hide progress bar
  try {
    await win.webContents.executeJavaScript(`
      (() => {
        const cancelBtn = document.getElementById('cancel-operation-btn');
        if (cancelBtn) cancelBtn.click();
      })()
    `);
    await new Promise(r => setTimeout(r, 300));
    image = await win.capturePage();
    fs.writeFileSync(path.join(brainDir, 'dawn_cancelled_hidden_preview.png'), image.toPNG());
  } catch (err) {
    console.error('Step 5 error:', err);
  }

  console.log('ALL_SCREENSHOTS_SUCCESS');
  app.exit(0);
});
