const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  isElectron: true,

  // Window controls
  minimize: () => ipcRenderer.send('window-minimize'),
  maximize: () => ipcRenderer.send('window-maximize'),
  close: () => ipcRenderer.send('window-close'),

  // Game detection & Preflight validation
  selectGameFolder: () => ipcRenderer.invoke('select-game-folder'),
  validateGameFolder: (folderPath) => ipcRenderer.invoke('validate-game-folder', folderPath),
  detectGameFolder: () => ipcRenderer.invoke('detect-game-folder'),
  validatePreflight: (folderPath) => ipcRenderer.invoke('validate-preflight', folderPath),
  getInstallerConstants: () => ipcRenderer.invoke('get-installer-constants'),

  // DepotDownloader & Steam Depot operations
  startDepotDownload: (options) => ipcRenderer.invoke('start-depot-download', options),
  sendConsoleInput: (text) => ipcRenderer.invoke('send-console-input', text),
  cancelDepotDownload: () => ipcRenderer.invoke('cancel-depot-download'),

  // Dawn Mod Installation & Maintenance
  installDawn: (gameRoot) => ipcRenderer.invoke('install-dawn', gameRoot),
  restoreDawn: (gameRoot) => ipcRenderer.invoke('restore-dawn', gameRoot),
  clearCache: (gameRoot) => ipcRenderer.invoke('clear-cache', gameRoot),
  launchGame: (gameRoot) => ipcRenderer.invoke('launch-game', gameRoot),
  openFolder: (folderPath) => ipcRenderer.invoke('open-folder', folderPath),
  openDocs: () => ipcRenderer.invoke('open-docs'),

  // Real-time events from backend
  onLog: (callback) => ipcRenderer.on('installer:log', (event, data) => callback(data)),
  onProgress: (callback) => ipcRenderer.on('installer:progress', (event, data) => callback(data)),
  onDepotOutput: (callback) => ipcRenderer.on('depot:output', (event, text) => callback(text)),
  onDepotProgress: (callback) => ipcRenderer.on('depot:progress', (event, data) => callback(data)),
  onSteamGuardNeeded: (callback) => ipcRenderer.on('depot:steam-guard', (event, data) => callback(data)),
  onSteamQrCode: (callback) => ipcRenderer.on('depot:qr-code', (event, svg) => callback(svg))
});
