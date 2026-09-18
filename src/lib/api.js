import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

const isTauri = typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__);

export const api = {
  isTauri,

  async selectGameFolder() {
    if (isTauri) {
      return await invoke('select_game_folder');
    }
    return 'C:\\Games\\Destiny 2';
  },

  async getLatestDawnVersion() {
    if (isTauri) {
      return await invoke('get_latest_dawn_version');
    }
    return '0.1.3';
  },

  async getInstallerConstants() {
    if (isTauri) {
      return await invoke('get_installer_constants');
    }
    return {
      languages: [
        { name: 'English', code: 'english', depot_id: 1085662, manifest_id: '2210332166360342287' },
        { name: 'French', code: 'french', depot_id: 1085663, manifest_id: '2934940253687559290' },
        { name: 'German', code: 'german', depot_id: 1085664, manifest_id: '2207989571290186153' },
        { name: 'Italian', code: 'italian', depot_id: 1085665, manifest_id: '6668232053215128229' },
        { name: 'Japanese', code: 'japanese', depot_id: 1085666, manifest_id: '7430022397683116838' },
        { name: 'Portuguese (Brazil)', code: 'brazilian', depot_id: 1085667, manifest_id: '9037238175838085860' },
        { name: 'Spanish (Spain)', code: 'spanish', depot_id: 1085668, manifest_id: '3424833900894552134' },
        { name: 'Russian', code: 'russian', depot_id: 1085669, manifest_id: '4539277942371480381' },
        { name: 'Polish', code: 'polish', depot_id: 1085670, manifest_id: '6407581507105256731' },
        { name: 'Chinese (Simplified)', code: 'schinese', depot_id: 1085671, manifest_id: '4397663774546719308' },
        { name: 'Chinese (Traditional)', code: 'tchinese', depot_id: 1085672, manifest_id: '3906738704604711877' },
        { name: 'Spanish (Latin America)', code: 'latam', depot_id: 1085673, manifest_id: '4773170998099699561' },
        { name: 'Korean', code: 'koreana', depot_id: 1085674, manifest_id: '7148196199569436690' },
      ],
      required_free_bytes: 110 * 1024 * 1024 * 1024,
      base_depot: { depot_id: 1085661, manifest_id: '7180122903232116872' },
    };
  },

  async validatePreflight(folderPath) {
    if (isTauri) {
      return await invoke('validate_preflight', { folderPath });
    }
    return {
      valid: true,
      path: folderPath,
      exists: true,
      writeable: true,
      has_game: true,
      free_bytes: 150000000000,
      free_gb: 139.7,
      required_bytes: 118111600640,
      required_gb: 110,
      has_enough_space: true,
      warning: null,
      error: null,
    };
  },

  async validateGameFolder(folderPath) {
    if (isTauri) {
      return await invoke('validate_game_folder', { folderPath });
    }
    return { valid: true, has_packages: true, message: 'Valid Destiny 2 installation found' };
  },

  async detectGameFolder() {
    if (isTauri) {
      return await invoke('detect_game_folder');
    }
    return null;
  },

  async startDepotDownload({ installRoot, languageCode, authMethod, steamUsername, steamPassword }) {
    if (isTauri) {
      return await invoke('start_depot_download', {
        installRoot,
        languageCode,
        authMethod,
        steamUsername,
        steamPassword,
      });
    }
    return { success: true, message: 'Mock download started' };
  },

  async cancelDepotDownload() {
    if (isTauri) {
      return await invoke('cancel_depot_download');
    }
    return true;
  },

  async sendConsoleInput(text) {
    if (isTauri) {
      return await invoke('send_console_input', { text });
    }
    return true;
  },

  async installDawn(gameRoot) {
    if (isTauri) {
      return await invoke('install_dawn', { gameRoot });
    }
    return { success: true, message: 'Dawn installed successfully!' };
  },

  async restoreDawn(gameRoot) {
    if (isTauri) {
      return await invoke('restore_dawn', { gameRoot });
    }
    return { success: true, message: 'Restoration complete' };
  },

  async uninstallDawn(gameRoot) {
    if (isTauri) {
      return await invoke('uninstall_dawn', { gameRoot });
    }
    return { success: true, message: 'Dawn uninstalled successfully' };
  },

  async clearCache(gameRoot) {
    if (isTauri) {
      return await invoke('clear_cache', { gameRoot });
    }
    return { success: true, count: 0, message: 'Cleared cache' };
  },

  async launchGame(gameRoot, languageCode = 'english') {
    if (isTauri) {
      return await invoke('launch_game', { gameRoot, languageCode });
    }
    return { success: true, message: 'Game launched' };
  },

  async getPlatform() {
    if (isTauri) {
      try {
        return await invoke('get_platform');
      } catch (_) {}
    }
    const ua = (typeof navigator !== 'undefined' ? (navigator.userAgent || navigator.platform || '') : '').toLowerCase();
    if (ua.includes('linux')) return 'linux';
    if (ua.includes('win')) return 'windows';
    if (ua.includes('mac')) return 'macos';
    return 'unknown';
  },

  async openFolder(targetPath) {
    if (isTauri) {
      return await invoke('open_folder', { targetPath });
    }
    return true;
  },

  async openDocs() {
    if (isTauri) {
      return await invoke('open_docs');
    }
    return true;
  },

  // Debug Logging
  async getDebugLogs() {
    if (isTauri) {
      return await invoke('get_debug_logs');
    }
    return [];
  },

  async openLogFile() {
    if (isTauri) {
      return await invoke('open_log_file');
    }
    return true;
  },

  async clearDebugLogs() {
    if (isTauri) {
      return await invoke('clear_debug_logs');
    }
    return true;
  },

  // Window actions
  async minimizeWindow() {
    if (isTauri) {
      try {
        await invoke('minimize_window');
      } catch (_) {
        await getCurrentWindow().minimize();
      }
    }
  },

  async maximizeWindow() {
    if (isTauri) {
      try {
        const win = getCurrentWindow();
        const isMax = await win.isMaximized();
        if (isMax) {
          await win.unmaximize();
        } else {
          await win.maximize();
        }
      } catch (_) {}
    }
  },

  async closeWindow() {
    if (isTauri) {
      try {
        await invoke('close_window');
      } catch (_) {
        await invoke('cancel_depot_download');
        await getCurrentWindow().close();
      }
    }
  },

  // Event listeners
  onDepotOutput(cb) {
    if (isTauri) {
      return listen('depot:output', (event) => cb(event.payload));
    }
    return () => {};
  },

  onDepotProgress(cb) {
    if (isTauri) {
      return listen('depot:progress', (event) => cb(event.payload));
    }
    return () => {};
  },

  onDepotSteamGuard(cb) {
    if (isTauri) {
      return listen('depot:steam-guard', (event) => cb(event.payload));
    }
    return () => {};
  },

  onDepotQrCode(cb) {
    if (isTauri) {
      return listen('depot:qr-code', (event) => cb(event.payload));
    }
    return () => {};
  },

  onAuthError(cb) {
    if (isTauri) {
      return listen('depot:auth-error', (event) => cb(event.payload));
    }
    return () => {};
  },

  onDebugLog(cb) {
    if (isTauri) {
      return listen('debug:log', (event) => cb(event.payload));
    }
    return () => {};
  },

  onInstallerProgress(cb) {
    if (isTauri) {
      return listen('installer:progress', (event) => cb(event.payload));
    }
    return () => {};
  },
};
