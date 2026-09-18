const { app, BrowserWindow } = require('electron');
const path = require('path');
const fs = require('fs');

app.whenReady().then(async () => {
  const win = new BrowserWindow({
    width: 1040,
    height: 585,
    show: false,
    webPreferences: {
      offscreen: true,
      preload: path.join(__dirname, 'electron', 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  await win.loadFile(path.join(__dirname, 'dist', 'index.html'));
  await new Promise(r => setTimeout(r, 1200));

  await win.webContents.executeJavaScript(`
    const card = document.getElementById('install-progress-card');
    card.classList.add('active');
    const consoleElem = document.getElementById('installer-console');
    consoleElem.classList.add('open');
    document.getElementById('progress-title').textContent = 'Downloading Destiny 2 & Installing Dawn';
    document.getElementById('progress-step-desc').textContent = 'Depots: 1085661 (Base) + Language (english)';
    document.getElementById('progress-percent').textContent = '42%';
    document.getElementById('progress-bar-fill').style.width = '42%';
    document.getElementById('progress-details').textContent = 'Downloading Depot 1085661: 42.1% (15.2 GB / 36.5 GB)';
    const logs = document.getElementById('console-logs');
    logs.textContent = '[INITIALIZING] DepotDownloader 3.4.0 verified (SHA-256 OK)\\n' +
      '[AUTH] Scan QR code with Steam Mobile App to authenticate:\\n' +
      '  ██████████████  ████  ██████████████  \\n' +
      '  ██          ██  ██    ██          ██  \\n' +
      '  ██  ██████  ██  ████  ██  ██████  ██  \\n' +
      '  ██  ██████  ██    ██  ██  ██████  ██  \\n' +
      '  ██████████████  ████  ██████████████  \\n' +
      '[DEPOT] Connected to Steam CDN. Downloading Base Depot 1085661...\\n' +
      '42.1% (15.2 GB / 36.5 GB) - 28.4 MB/s';
  `);

  await new Promise(r => setTimeout(r, 600));

  const image = await win.capturePage();
  fs.writeFileSync('C:/Users/Matteo/.gemini/antigravity/brain/dcd6b415-40d1-4e48-9211-ded1dcbcdaf9/dawn_standalone_installer_downloading_preview.png', image.toPNG());
  console.log('PROGRESS_SCREENSHOT_SUCCESS');
  app.exit(0);
});
