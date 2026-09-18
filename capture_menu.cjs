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
    document.getElementById('install-menu').classList.add('open');
    document.getElementById('install-arrow-btn').classList.add('active');
  `);

  await new Promise(r => setTimeout(r, 400));

  const image = await win.capturePage();
  fs.writeFileSync('C:/Users/Matteo/.gemini/antigravity/brain/dcd6b415-40d1-4e48-9211-ded1dcbcdaf9/dawn_standalone_installer_menu_preview.png', image.toPNG());
  console.log('MENU_SCREENSHOT_SUCCESS');
  app.exit(0);
});
