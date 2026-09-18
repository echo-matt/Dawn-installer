const { app, BrowserWindow, session } = require('electron');
const path = require('path');
const fs = require('fs');

app.whenReady().then(async () => {
  await session.defaultSession.clearStorageData();

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

  // Load about:blank first, clear localStorage, then load index.html pristine
  await win.loadURL('about:blank');
  await session.defaultSession.clearStorageData();

  await win.loadFile(path.join(__dirname, 'dist', 'index.html'));
  await win.webContents.executeJavaScript(`
    localStorage.clear();
    // re-trigger initial preflight check with empty storage
    window.dispatchEvent(new Event('DOMContentLoaded'));
  `);
  await new Promise(r => setTimeout(r, 1200));

  const image = await win.capturePage();
  fs.writeFileSync('C:/Users/Matteo/.gemini/antigravity/brain/dcd6b415-40d1-4e48-9211-ded1dcbcdaf9/dawn_first_launch_preview.png', image.toPNG());
  console.log('PRISTINE_FIRST_LAUNCH_SUCCESS');
  app.exit(0);
});
