const { app, BrowserWindow } = require('electron');
const path = require('path');
const fs = require('fs');

app.whenReady().then(async () => {
  const win = new BrowserWindow({
    width: 1040,
    height: 585,
    show: false,
    webPreferences: { offscreen: true }
  });

  await win.loadFile(path.join(__dirname, 'dist', 'index.html'));
  await new Promise(r => setTimeout(r, 600));

  // Set mock path and run installation
  await win.webContents.executeJavaScript(`
    window.electronApp = window.electronApp || {};
    // Trigger install progress directly
    const appInstance = window.__dawnApp;
    document.getElementById('install-progress-card').classList.add('active');
    document.getElementById('progress-bar-fill').style.width = '64%';
    document.getElementById('progress-percent').textContent = '64%';
    document.getElementById('progress-title').textContent = 'Installing Dawn...';
    document.getElementById('progress-details').textContent = 'Deploying mission scripts & configuration profiles...';
    document.getElementById('installer-console').classList.add('open');
    document.getElementById('toggle-log-btn').textContent = 'Hide Log';
    
    const logs = document.getElementById('console-logs');
    logs.innerHTML = '<div class="console-line">[INFO] Located game directory: D:\\\\Games\\\\Destiny 2</div>' +
                     '<div class="console-line">[BACKUP] Creating backup at .dawn\\\\backup\\\\20260918-120000</div>' +
                     '<div class="console-line success">[DEPLOY] Copied mission Lua & JSON scripts</div>' +
                     '<div class="console-line">[CONFIG] Deployed default settings, hud, and player profiles</div>' +
                     '<div class="console-line">[STATUS] Deploying launcher commands...</div>';
  `);
  await new Promise(r => setTimeout(r, 600));

  const image = await win.capturePage();
  fs.writeFileSync(path.join(__dirname, 'screenshot_installing.png'), image.toPNG());
  console.log('INSTALLING_SCREENSHOT_SUCCESS');
  app.exit(0);
});
