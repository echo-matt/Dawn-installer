const { app, BrowserWindow, ipcMain, dialog, shell } = require('electron');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');
const depotService = require('./depot-service.cjs');

let mainWindow;
const DAWN_REPO_PATH = 'D:\\Documents\\VSCode stuff\\Dawn';
const BUNDLED_PAYLOAD_DIR = path.join(__dirname, '..', 'bundle', 'dawn-release');

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1040,
    height: 585,
    minWidth: 840,
    minHeight: 520,
    frame: false,
    transparent: false,
    backgroundColor: '#040b1e',
    titleBarStyle: 'hidden',
    titleBarOverlay: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: false
    },
    show: false
  });

  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    const indexPath = path.join(__dirname, '../dist/index.html');
    mainWindow.loadFile(indexPath).catch(() => {
      mainWindow.loadFile(path.join(__dirname, '../index.html'));
    });
  }

  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

  // Window controls
  ipcMain.on('window-minimize', () => {
    if (mainWindow) mainWindow.minimize();
  });

  ipcMain.on('window-maximize', () => {
    if (mainWindow) {
      if (mainWindow.isMaximized()) {
        mainWindow.unmaximize();
      } else {
        mainWindow.maximize();
      }
    }
  });

  ipcMain.on('window-close', () => {
    depotService.cancelDownload();
    if (mainWindow) mainWindow.close();
  });

  // Folder selection
  ipcMain.handle('select-game-folder', async () => {
    const result = await dialog.showOpenDialog(mainWindow, {
      title: 'Select Destiny 2 Game Directory',
      properties: ['openDirectory', 'createDirectory']
    });
    if (result.canceled || result.filePaths.length === 0) {
      return null;
    }
    return result.filePaths[0];
  });

  // Preflight validation
  ipcMain.handle('validate-preflight', async (event, folderPath) => {
    return depotService.validatePreflight(folderPath);
  });

  // Installer Constants
  ipcMain.handle('get-installer-constants', async () => {
    return depotService.getConstants();
  });

  // Validate game folder (fast check)
  ipcMain.handle('validate-game-folder', async (event, folderPath) => {
    if (!folderPath || !fs.existsSync(folderPath)) {
      return { valid: false, message: 'Directory does not exist' };
    }
    const exePath = path.join(folderPath, 'destiny2.exe');
    if (!fs.existsSync(exePath)) {
      return { valid: false, message: 'destiny2.exe not found in this folder' };
    }
    const packagesPath = path.join(folderPath, 'packages');
    const hasPackages = fs.existsSync(packagesPath);
    return {
      valid: true,
      hasPackages,
      exePath,
      message: hasPackages ? 'Valid Destiny 2 installation found' : 'destiny2.exe found (packages folder missing)'
    };
  });

  // Auto-detect game folder
  ipcMain.handle('detect-game-folder', async () => {
    const drives = ['C:', 'D:', 'E:', 'F:', 'G:'];
    const candidates = [
      'Dawn',
      'Destiny 2',
      'Games\\Destiny 2',
      'Games\\Dawn',
      'SteamLibrary\\steamapps\\common\\Destiny 2',
      'Program Files (x86)\\Steam\\steamapps\\common\\Destiny 2',
      'Program Files\\Steam\\steamapps\\common\\Destiny 2'
    ];

    for (const drive of drives) {
      for (const rel of candidates) {
        const testPath = path.join(drive, '\\', rel);
        try {
          if (fs.existsSync(path.join(testPath, 'destiny2.exe'))) {
            return testPath;
          }
        } catch (_) {}
      }
    }
    return null;
  });

  // DepotDownloader Steam download
  ipcMain.handle('start-depot-download', async (event, { installRoot, languageCode, authMethod = 'qr', steamUsername, steamPassword }) => {
    const sendOutput = (text) => {
      if (mainWindow) mainWindow.webContents.send('depot:output', text);
    };

    const sendProgress = (percent, status) => {
      if (mainWindow) mainWindow.webContents.send('depot:progress', { percent, status });
    };

    const sendStatus = (statusText) => {
      if (mainWindow) mainWindow.webContents.send('installer:progress', { percent: 0, status: statusText });
    };

    const sendSteamGuard = (data) => {
      if (mainWindow) mainWindow.webContents.send('depot:steam-guard', data);
    };

    const sendQrCode = (svgMarkup) => {
      if (mainWindow) mainWindow.webContents.send('depot:qr-code', svgMarkup);
    };

    try {
      sendOutput(`\r\n[INITIALIZING] Preparing download to: ${installRoot}\r\n`);
      const result = await depotService.downloadDepots({
        installRoot,
        languageCode,
        authMethod,
        steamUsername,
        steamPassword,
        onOutput: sendOutput,
        onProgress: sendProgress,
        onStatus: sendStatus,
        onSteamGuard: sendSteamGuard,
        onQrCode: sendQrCode
      });

      if (result.cancelled) {
        sendOutput('\r\n[CANCELLED] Download was cancelled by user.\r\n');
        return { success: false, cancelled: true };
      }

      // Automatically install bundled Dawn mod after game depots download
      sendOutput('\r\n[INSTALLING DAWN] Installing bundled Dawn mod payload over downloaded game...\r\n');
      await depotService.installBundledDawn(installRoot, sendOutput, sendProgress);

      return { success: true, message: 'Game & Dawn installed successfully!' };
    } catch (err) {
      sendOutput(`\r\n[ERROR] ${err.message}\r\n`);
      return { success: false, error: err.message };
    }
  });

  ipcMain.handle('send-console-input', async (event, text) => {
    return depotService.sendConsoleInput(text);
  });

  ipcMain.handle('cancel-depot-download', async () => {
    depotService.cancelDownload();
    return true;
  });

  // Install Dawn using bundled Install-Dawn.ps1
  ipcMain.handle('install-dawn', async (event, gameRoot) => {
    return new Promise((resolve) => {
      function sendLog(msg, type = 'info') {
        if (mainWindow) mainWindow.webContents.send('installer:log', { message: msg, type });
      }

      function sendProgress(percent, status) {
        if (mainWindow) mainWindow.webContents.send('installer:progress', { percent, status });
      }

      sendLog(`Starting Dawn installation to: ${gameRoot}`);
      sendProgress(5, 'Preparing Dawn deployment...');

      // Prefer bundled Install-Dawn.ps1
      const bundledScript = path.join(BUNDLED_PAYLOAD_DIR, 'Install-Dawn.ps1');
      const repoScript = path.join(DAWN_REPO_PATH, 'tools', 'install', 'Install-Dawn.ps1');
      const activeScript = fs.existsSync(bundledScript) ? bundledScript : repoScript;
      const scriptCwd = fs.existsSync(bundledScript) ? BUNDLED_PAYLOAD_DIR : DAWN_REPO_PATH;

      if (fs.existsSync(activeScript)) {
        sendLog(`Executing installer engine: ${activeScript}`);
        const psArgs = [
          '-NoProfile',
          '-ExecutionPolicy', 'Bypass',
          '-File', activeScript,
          '-GameRoot', gameRoot
        ];

        const ps = spawn('powershell.exe', psArgs, { cwd: scriptCwd });

        ps.stdout.on('data', (data) => {
          const text = data.toString().trim();
          if (text) {
            sendLog(text);
            if (text.includes('Game install:')) sendProgress(20, 'Located game install...');
            else if (text.includes('Backing up') || text.includes('backup')) sendProgress(40, 'Creating safety backup...');
            else if (text.includes('scripts') || text.includes('Deploying') || text.includes('Copying')) sendProgress(70, 'Deploying mission scripts & configs...');
            else if (text.includes('settings.json')) sendProgress(90, 'Configuring arrival overrides...');
          }
        });

        ps.stderr.on('data', (data) => {
          const text = data.toString().trim();
          if (text) sendLog(text, 'warn');
        });

        ps.on('close', (code) => {
          if (code === 0) {
            sendLog('Installation completed successfully!', 'success');
            sendProgress(100, 'Dawn installed successfully!');
            resolve({ success: true, message: 'Installation completed successfully!' });
          } else {
            sendLog(`PowerShell script exited with code ${code}. Performing direct fallback copy...`, 'warn');
            try {
              performDirectDeployment(gameRoot, sendLog, sendProgress);
              resolve({ success: true, message: 'Direct deployment completed!' });
            } catch (err) {
              sendLog(`Direct deployment failed: ${err.message}`, 'error');
              resolve({ success: false, message: err.message });
            }
          }
        });

        ps.on('error', (err) => {
          sendLog(`PowerShell error: ${err.message}. Falling back to direct copy...`, 'warn');
          try {
            performDirectDeployment(gameRoot, sendLog, sendProgress);
            resolve({ success: true, message: 'Direct deployment completed!' });
          } catch (e) {
            resolve({ success: false, message: e.message });
          }
        });
      } else {
        try {
          performDirectDeployment(gameRoot, sendLog, sendProgress);
          resolve({ success: true, message: 'Direct deployment completed!' });
        } catch (err) {
          resolve({ success: false, message: err.message });
        }
      }
    });
  });

  // Restore Dawn backup
  ipcMain.handle('restore-dawn', async (event, gameRoot) => {
    return new Promise((resolve) => {
      function sendLog(msg, type = 'info') {
        if (mainWindow) mainWindow.webContents.send('installer:log', { message: msg, type });
      }

      const bundledScript = path.join(BUNDLED_PAYLOAD_DIR, 'Install-Dawn.ps1');
      const repoScript = path.join(DAWN_REPO_PATH, 'tools', 'install', 'Install-Dawn.ps1');
      const activeScript = fs.existsSync(bundledScript) ? bundledScript : repoScript;
      const scriptCwd = fs.existsSync(bundledScript) ? BUNDLED_PAYLOAD_DIR : DAWN_REPO_PATH;

      if (fs.existsSync(activeScript)) {
        sendLog('Running Restore via Dawn installer engine...');
        const ps = spawn('powershell.exe', [
          '-NoProfile',
          '-ExecutionPolicy', 'Bypass',
          '-File', activeScript,
          '-GameRoot', gameRoot,
          '-Restore'
        ], { cwd: scriptCwd });

        ps.stdout.on('data', (d) => sendLog(d.toString().trim()));
        ps.stderr.on('data', (d) => sendLog(d.toString().trim(), 'warn'));

        ps.on('close', (code) => {
          resolve({ success: code === 0, message: code === 0 ? 'Restoration complete' : 'Restore script failed' });
        });
      } else {
        resolve({ success: false, message: 'Installer script not found' });
      }
    });
  });

  // Clear Dawn Cache
  ipcMain.handle('clear-cache', async (event, gameRoot) => {
    const targets = [
      path.join(gameRoot, 'Dawn', 'cache'),
      path.join(gameRoot, 'bin', 'x64', 'Dawn', 'cache')
    ];
    let cleared = 0;
    for (const t of targets) {
      if (fs.existsSync(t)) {
        const files = fs.readdirSync(t);
        for (const f of files) {
          if (f.endsWith('.bin')) {
            try {
              fs.unlinkSync(path.join(t, f));
              cleared++;
            } catch (_) {}
          }
        }
      }
    }
    return { success: true, count: cleared, message: `Cleared ${cleared} cache files` };
  });

  // Launch Game with DAWN_FOREST_BASELINE=1
  ipcMain.handle('launch-game', async (event, gameRoot) => {
    const cmdPath = path.join(gameRoot, 'launch-destiny.cmd');
    const exePath = path.join(gameRoot, 'destiny2.exe');

    const targetToRun = fs.existsSync(cmdPath) ? cmdPath : exePath;
    if (!fs.existsSync(targetToRun)) {
      return { success: false, message: 'Game executable not found' };
    }

    const env = { ...process.env, DAWN_FOREST_BASELINE: '1' };
    spawn(targetToRun, [], { cwd: gameRoot, detached: true, env, stdio: 'ignore' }).unref();
    return { success: true, message: 'Game launched' };
  });

  // Open directory in Explorer
  ipcMain.handle('open-folder', async (event, targetPath) => {
    if (targetPath && fs.existsSync(targetPath)) {
      shell.openPath(targetPath);
      return true;
    }
    return false;
  });

  // Open Documentation
  ipcMain.handle('open-docs', async () => {
    const readme = path.join(BUNDLED_PAYLOAD_DIR, 'READ-ME.txt');
    if (fs.existsSync(readme)) {
      shell.openPath(readme);
    } else {
      shell.openExternal('https://github.com/isinternets/Dawn');
    }
    return true;
  });
}

function copyDirSync(src, dest) {
  if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
  const entries = fs.readdirSync(src, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirSync(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

function performDirectDeployment(gameRoot, sendLog, sendProgress) {
  sendLog('Deploying runtime resources from bundled Dawn payload...');
  sendProgress(40, 'Deploying mission scripts & configs...');

  const payloadSrc = path.join(BUNDLED_PAYLOAD_DIR, 'payload');
  const targetDawn1 = path.join(gameRoot, 'Dawn');
  const targetDawn2 = path.join(gameRoot, 'bin', 'x64', 'Dawn');

  if (fs.existsSync(payloadSrc)) {
    copyDirSync(payloadSrc, gameRoot);
    if (fs.existsSync(path.join(payloadSrc, 'Dawn'))) {
      copyDirSync(path.join(payloadSrc, 'Dawn'), targetDawn2);
    }
    sendProgress(100, 'Dawn files deployed successfully!');
    sendLog('Deployment finished successfully.', 'success');
  } else {
    throw new Error('Bundled payload directory missing');
  }
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
