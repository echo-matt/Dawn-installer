const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');
const { spawn, execFile } = require('child_process');

const APP_CONSTANTS = {
  STEAM_APP_ID: 1085660,
  GAME_EXECUTABLE: 'destiny2.exe',
  BASE_DEPOT: {
    depotId: 1085661,
    manifestId: '7180122903232116872'
  },
  REQUIRED_FREE_BYTES: 110 * 1024 * 1024 * 1024, // 110 GiB
  DEPOT_DOWNLOADER: {
    version: '3.4.0',
    url: 'https://github.com/SteamRE/DepotDownloader/releases/download/DepotDownloader_3.4.0/DepotDownloader-windows-x64.zip',
    sha256: '41c9e9f0df54b3ad02e67a11726756e5c73283bd7c2e1b04acfa5ae4c2ed3767',
    archiveName: 'DepotDownloader-windows-x64.zip'
  },
  LANGUAGES: [
    { name: 'English', code: 'english', depotId: 1085662, manifestId: '2210332166360342287' },
    { name: 'French', code: 'french', depotId: 1085663, manifestId: '2934940253687559290' },
    { name: 'German', code: 'german', depotId: 1085664, manifestId: '2207989571290186153' },
    { name: 'Italian', code: 'italian', depotId: 1085665, manifestId: '6668232053215128229' },
    { name: 'Japanese', code: 'japanese', depotId: 1085666, manifestId: '7430022397683116838' },
    { name: 'Portuguese (Brazil)', code: 'brazilian', depotId: 1085667, manifestId: '9037238175838085860' },
    { name: 'Spanish (Spain)', code: 'spanish', depotId: 1085668, manifestId: '3424833900894552134' },
    { name: 'Russian', code: 'russian', depotId: 1085669, manifestId: '4539277942371480381' },
    { name: 'Polish', code: 'polish', depotId: 1085670, manifestId: '6407581507105256731' },
    { name: 'Chinese (Simplified)', code: 'schinese', depotId: 1085671, manifestId: '4397663774546719308' },
    { name: 'Chinese (Traditional)', code: 'tchinese', depotId: 1085672, manifestId: '3906738704604711877' },
    { name: 'Spanish (Latin America)', code: 'latam', depotId: 1085673, manifestId: '4773170998099699561' },
    { name: 'Korean', code: 'koreana', depotId: 1085674, manifestId: '7148196199569436690' }
  ]
};

class DepotService {
  constructor() {
    this.activeProcess = null;
    this.isCancelled = false;
    this.toolsRoot = path.join(
      process.env.LOCALAPPDATA || process.env.APPDATA || path.join(process.env.USERPROFILE || 'C:\\', 'AppData', 'Local'),
      'DawnInstaller',
      'tools',
      'DepotDownloader',
      APP_CONSTANTS.DEPOT_DOWNLOADER.version
    );
  }

  getConstants() {
    return {
      languages: APP_CONSTANTS.LANGUAGES,
      requiredFreeBytes: APP_CONSTANTS.REQUIRED_FREE_BYTES,
      baseDepot: APP_CONSTANTS.BASE_DEPOT
    };
  }

  validatePreflight(targetPath) {
    if (!targetPath || typeof targetPath !== 'string' || !targetPath.trim()) {
      return { valid: false, error: 'Please select a destination folder.' };
    }

    const resolved = path.resolve(targetPath.trim());
    const root = path.parse(resolved).root;

    // Must not be the drive root itself (e.g. C:\)
    if (resolved.toLowerCase() === root.toLowerCase()) {
      return {
        valid: false,
        error: 'Choose a subfolder on the drive (e.g. D:\\Games\\Destiny2), not the root drive itself.'
      };
    }

    let exists = false;
    try {
      exists = fs.existsSync(resolved);
      if (!exists) {
        fs.mkdirSync(resolved, { recursive: true });
        exists = true;
      }
    } catch (err) {
      return { valid: false, error: `Cannot access folder: ${err.message}` };
    }

    // Check write permissions with temporary probe file
    let writeable = false;
    const probeFile = path.join(resolved, `.dawn_write_test_${Date.now()}_${Math.random().toString(36).substring(2, 8)}.tmp`);
    try {
      fs.writeFileSync(probeFile, 'dawn_test_probe_write');
      fs.unlinkSync(probeFile);
      writeable = true;
    } catch (err) {
      writeable = false;
      return {
        valid: false,
        error: `Folder is write-protected or lacks permissions: ${err.message}`
      };
    }

    // Check free disk space via statfsSync
    let freeBytes = 0;
    try {
      const stats = fs.statfsSync(resolved);
      freeBytes = stats.bavail * stats.bsize;
    } catch (err) {
      freeBytes = 0;
    }

    const freeGB = Number((freeBytes / (1024 ** 3)).toFixed(2));
    const requiredGB = Number((APP_CONSTANTS.REQUIRED_FREE_BYTES / (1024 ** 3)).toFixed(0));
    const hasGame = fs.existsSync(path.join(resolved, APP_CONSTANTS.GAME_EXECUTABLE));
    const hasEnoughSpace = hasGame || freeBytes >= APP_CONSTANTS.REQUIRED_FREE_BYTES;

    return {
      valid: true,
      path: resolved,
      exists,
      writeable,
      hasGame,
      freeBytes,
      freeGB,
      requiredBytes: APP_CONSTANTS.REQUIRED_FREE_BYTES,
      requiredGB,
      hasEnoughSpace,
      warning: !hasEnoughSpace
        ? `Low disk space: ${freeGB} GB available. Fresh install requires ~${requiredGB} GB.`
        : null
    };
  }

  async ensureDepotDownloader(onStatus, onProgress) {
    const exePath = path.join(this.toolsRoot, 'DepotDownloader.exe');
    if (fs.existsSync(exePath)) {
      return exePath;
    }

    fs.mkdirSync(this.toolsRoot, { recursive: true });
    const archivePath = path.join(this.toolsRoot, APP_CONSTANTS.DEPOT_DOWNLOADER.archiveName);

    if (onStatus) onStatus('Downloading DepotDownloader 3.4.0...');

    // Download archive with redirect support
    await new Promise((resolve, reject) => {
      const downloadWithRedirect = (targetUrl) => {
        https.get(targetUrl, { headers: { 'User-Agent': 'DawnInstaller/1.0' } }, (response) => {
          if (response.statusCode === 301 || response.statusCode === 302 || response.statusCode === 307) {
            downloadWithRedirect(response.headers.location);
            return;
          }

          if (response.statusCode !== 200) {
            reject(new Error(`Failed to download DepotDownloader: HTTP ${response.statusCode}`));
            return;
          }

          const totalBytes = parseInt(response.headers['content-length'] || '0', 10);
          let downloadedBytes = 0;
          const fileStream = fs.createWriteStream(archivePath);

          response.on('data', (chunk) => {
            downloadedBytes += chunk.length;
            if (totalBytes > 0 && onProgress) {
              const pct = Math.round((downloadedBytes / totalBytes) * 100);
              onProgress(pct, `Downloading DepotDownloader: ${pct}% (${(downloadedBytes / 1048576).toFixed(1)}MB / ${(totalBytes / 1048576).toFixed(1)}MB)`);
            }
          });

          response.pipe(fileStream);
          fileStream.on('finish', () => {
            fileStream.close(resolve);
          });
          fileStream.on('error', reject);
        }).on('error', reject);
      };

      downloadWithRedirect(APP_CONSTANTS.DEPOT_DOWNLOADER.url);
    });

    // Check SHA-256 checksum
    if (onStatus) onStatus('Verifying DepotDownloader SHA-256 integrity...');
    const fileBuffer = fs.readFileSync(archivePath);
    const actualHash = crypto.createHash('sha256').update(fileBuffer).digest('hex').toLowerCase();
    const expectedHash = APP_CONSTANTS.DEPOT_DOWNLOADER.sha256.toLowerCase();

    if (actualHash !== expectedHash) {
      try { fs.unlinkSync(archivePath); } catch (_) {}
      throw new Error(`DepotDownloader checksum mismatch! Expected ${expectedHash}, got ${actualHash}`);
    }

    // Extract archive using native tar
    if (onStatus) onStatus('Extracting DepotDownloader...');
    await new Promise((resolve, reject) => {
      execFile('tar', ['-xf', archivePath, '-C', this.toolsRoot], (err) => {
        if (err) {
          // Fallback to powershell Expand-Archive
          execFile('powershell', ['-Command', `Expand-Archive -LiteralPath '${archivePath}' -DestinationPath '${this.toolsRoot}' -Force`], (psErr) => {
            if (psErr) reject(new Error(`Extraction failed: ${psErr.message}`));
            else resolve();
          });
        } else {
          resolve();
        }
      });
    });

    // Clean up zip archive
    try { fs.unlinkSync(archivePath); } catch (_) {}

    if (!fs.existsSync(exePath)) {
      throw new Error('DepotDownloader.exe not found after extraction.');
    }

    if (onStatus) onStatus('DepotDownloader 3.4.0 ready.');
    return exePath;
  }

  sendConsoleInput(text) {
    if (this.activeProcess && this.activeProcess.stdin && !this.activeProcess.stdin.destroyed) {
      this.activeProcess.stdin.write(text.trim() + '\r\n');
      return true;
    }
    return false;
  }

  cancelDownload() {
    this.isCancelled = true;
    if (this.activeProcess) {
      try {
        // Kill the whole process tree on Windows
        execFile('taskkill', ['/PID', this.activeProcess.pid.toString(), '/T', '/F'], () => {});
      } catch (_) {
        try { this.activeProcess.kill(); } catch (_) {}
      }
      this.activeProcess = null;
    }
  }

  parseQrFromBuffer(rawBuffer) {
    const str = rawBuffer.toString('binary');
    const marker = 'Use the Steam Mobile App to sign in with this QR code:';
    const idx = str.lastIndexOf(marker);
    if (idx === -1) return null;

    const after = str.slice(idx + marker.length).split('\r\n');
    let startIdx = -1;
    for (let i = 0; i < after.length; i++) {
      if (after[i].length >= 74) {
        startIdx = i;
        break;
      }
    }
    if (startIdx === -1 || after.length < startIdx + 37) return null;

    const qrLines = after.slice(startIdx, startIdx + 37);
    if (qrLines.length < 37) return null;

    const n = 37;
    const cellSize = 6;
    const size = n * cellSize;
    let rects = '';
    for (let r = 0; r < n; r++) {
      const line = qrLines[r];
      if (!line || line.length < 74) return null;
      for (let col = 0; col < 74; col += 2) {
        if (line.charCodeAt(col) === 219) { // 0xDB Full Block
          const x = (col / 2) * cellSize;
          const y = r * cellSize;
          rects += `<rect x="${x}" y="${y}" width="${cellSize}" height="${cellSize}" fill="#0f172a" />`;
        }
      }
    }
    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="${size}" height="${size}" shape-rendering="crispEdges"><rect width="${size}" height="${size}" fill="#ffffff" rx="10" />${rects}</svg>`;
  }

  async runDepotProcess(exePath, args, cwd, onOutput, onProgress, onSteamGuard, onQrCode, stepLabel) {
    return new Promise((resolve, reject) => {
      this.isCancelled = false;
      let lastErrorMessage = '';
      let binaryBuffer = Buffer.alloc(0);
      let capturedAccount = '';
      let lastSentQrSvg = '';

      const child = spawn(exePath, args, {
        cwd,
        windowsHide: true,
        stdio: ['pipe', 'pipe', 'pipe']
      });

      this.activeProcess = child;

      const handleData = (data) => {
        binaryBuffer = Buffer.concat([binaryBuffer, data]);
        const text = data.toString('utf8');
        if (onOutput) onOutput(text);

        // Check for QR code in binary stream
        if (onQrCode && binaryBuffer.toString('binary').includes('Use the Steam Mobile App')) {
          const qrSvg = this.parseQrFromBuffer(binaryBuffer);
          if (qrSvg && qrSvg !== lastSentQrSvg) {
            lastSentQrSvg = qrSvg;
            onQrCode(qrSvg);
          }
        }

        // Check for authenticated username
        const successUserMatch = text.match(/login with -username\s+(\S+)\s+-remember-password/i);
        if (successUserMatch) {
          capturedAccount = successUserMatch[1];
        }

        const lower = text.toLowerCase();

        // Check for Steam Guard code prompt
        const isEmailCode = lower.includes('email') || lower.includes('e-mail');
        const isCodePrompt =
          lower.includes('2 factor auth code') ||
          lower.includes('two-factor') ||
          lower.includes('2-factor') ||
          lower.includes('steam guard code') ||
          lower.includes('auth code') ||
          lower.includes('verification code') ||
          lower.includes('authenticator code') ||
          lower.includes('code sent to');

        const isMobilePushPrompt =
          lower.includes('approve the login') ||
          lower.includes('steam mobile app') ||
          lower.includes('mobile confirmation') ||
          lower.includes('mobile prompt') ||
          lower.includes('confirm on your phone') ||
          lower.includes('waiting for mobile');

        if (isCodePrompt) {
          if (onSteamGuard) {
            onSteamGuard({
              type: 'code',
              message: isEmailCode
                ? 'Enter the Steam Guard code sent to your email:'
                : 'Enter your Steam Guard Mobile Authenticator code:'
            });
          }
        } else if (isMobilePushPrompt) {
          if (onSteamGuard) {
            onSteamGuard({
              type: 'mobile_confirm',
              message: 'Please approve the login request on your Steam Mobile App'
            });
          }
        }

        // Catch specific error conditions
        if (text.includes('InvalidPassword') || lower.includes('invalid password')) {
          lastErrorMessage = 'Invalid Steam password. Please check your credentials.';
        } else if (text.includes('InvalidLoginAuthCode') || text.includes('TwoFactorCodeMismatch')) {
          lastErrorMessage = 'Incorrect Steam Guard code. Please try again.';
        } else if (text.includes('RateLimitExceeded')) {
          lastErrorMessage = 'Steam rate limit exceeded. Please wait a few minutes before trying again.';
        } else if (text.includes('AccountLogonDenied')) {
          lastErrorMessage = 'Steam access denied. Please verify your account access.';
        }

        const percentMatch = text.match(/(\d+(?:\.\d+)?)%/);
        if (percentMatch && onProgress) {
          const pct = Math.min(100, Math.max(0, parseFloat(percentMatch[1])));
          onProgress(pct, `${stepLabel}: ${pct.toFixed(1)}%`);
        }
      };

      child.stdout.on('data', handleData);
      child.stderr.on('data', handleData);

      child.on('error', (err) => {
        this.activeProcess = null;
        reject(err);
      });

      child.on('close', (code) => {
        this.activeProcess = null;
        if (this.isCancelled) {
          resolve({ success: false, cancelled: true });
        } else if (code === 0) {
          resolve({ success: true, code, accountName: capturedAccount });
        } else {
          reject(new Error(lastErrorMessage || `DepotDownloader error (code ${code}).`));
        }
      });
    });
  }

  clearSavedSessions(installRoot) {
    const candidateDirs = [
      this.toolsRoot,
      installRoot,
      path.join(installRoot, '.DepotDownloader'),
      path.join(this.toolsRoot, 'config')
    ];
    for (const dir of candidateDirs) {
      if (!dir || !fs.existsSync(dir)) continue;
      try {
        const entries = fs.readdirSync(dir);
        for (const entry of entries) {
          if (
            entry.startsWith('.DepotDownloader') ||
            entry.endsWith('.login') ||
            entry.endsWith('.key') ||
            entry === 'config'
          ) {
            const target = path.join(dir, entry);
            fs.rmSync(target, { recursive: true, force: true });
          }
        }
      } catch (_) {}
    }
  }

  async downloadDepots({
    installRoot,
    languageCode = 'english',
    authMethod = 'qr',
    steamUsername = '',
    steamPassword = '',
    onOutput,
    onProgress,
    onStatus,
    onSteamGuard,
    onQrCode
  }) {
    const preflight = this.validatePreflight(installRoot);
    if (!preflight.valid) {
      throw new Error(preflight.error);
    }

    // Explicitly purge previous/current sessions to guarantee prompt login without reusing saved state
    this.clearSavedSessions(installRoot);

    const exePath = await this.ensureDepotDownloader(onStatus, onProgress);

    const language = APP_CONSTANTS.LANGUAGES.find(l => l.code === languageCode) || APP_CONSTANTS.LANGUAGES[0];
    const baseDepot = APP_CONSTANTS.BASE_DEPOT;

    const commonBaseArgs = [
      '-app', APP_CONSTANTS.STEAM_APP_ID.toString(),
      '-dir', installRoot,
      '-os', 'windows',
      '-osarch', '64'
    ];

    let baseAuthArgs = [];
    if (authMethod === 'qr') {
      baseAuthArgs = ['-qr', '-remember-password'];
      if (steamUsername && steamUsername.trim()) {
        baseAuthArgs.unshift('-username', steamUsername.trim());
      }
    } else {
      baseAuthArgs = [
        '-username', steamUsername.trim(),
        '-password', steamPassword,
        '-remember-password'
      ];
    }

    // Step 1: Base Shared Depot (1085661)
    if (onStatus) onStatus(`Starting download of Destiny 2 Base Depot (${baseDepot.depotId})...`);
    if (onOutput) onOutput(`\r\n=== DOWNLOADING BASE DEPOT (${baseDepot.depotId}) ===\r\n`);
    
    const baseArgs = [
      ...commonBaseArgs,
      ...baseAuthArgs,
      '-depot', baseDepot.depotId.toString(),
      '-manifest', baseDepot.manifestId
    ];

    const baseResult = await this.runDepotProcess(
      exePath,
      baseArgs,
      installRoot,
      onOutput,
      (pct, text) => {
        if (onProgress) onProgress(Math.round(pct * 0.85), `[1/2] Base Depot: ${text}`);
      },
      onSteamGuard,
      onQrCode,
      'Base Depot'
    );

    if (baseResult.cancelled) return { cancelled: true };

    // Step 2: Language Depot
    if (onStatus) onStatus(`Starting download of ${language.name} Language Depot (${language.depotId})...`);
    if (onOutput) onOutput(`\r\n=== DOWNLOADING ${language.name.toUpperCase()} DEPOT (${language.depotId}) ===\r\n`);

    let langAuthArgs = [];
    const savedUser = baseResult.accountName || (steamUsername ? steamUsername.trim() : '');
    if (savedUser) {
      langAuthArgs = ['-username', savedUser, '-remember-password'];
    } else if (authMethod === 'credentials') {
      langAuthArgs = ['-username', steamUsername.trim(), '-password', steamPassword, '-remember-password'];
    }

    const langArgs = [
      ...commonBaseArgs,
      ...langAuthArgs,
      '-depot', language.depotId.toString(),
      '-manifest', language.manifestId
    ];

    const langResult = await this.runDepotProcess(
      exePath,
      langArgs,
      installRoot,
      onOutput,
      (pct, text) => {
        if (onProgress) onProgress(85 + Math.round(pct * 0.10), `[2/2] ${language.name} Depot: ${text}`);
      },
      onSteamGuard,
      null,
      `${language.name} Depot`
    );

    if (langResult.cancelled) return { cancelled: true };

    if (onStatus) onStatus('Game depot download complete!');
    if (onProgress) onProgress(95, 'Game files downloaded successfully.');
    return { success: true };
  }

  async installBundledDawn(installRoot, onOutput, onProgress) {
    const bundledDir = path.join(__dirname, '..', 'bundle', 'dawn-release');
    const psScript = path.join(bundledDir, 'Install-Dawn.ps1');

    if (!fs.existsSync(psScript)) {
      throw new Error(`Bundled installer script not found at ${psScript}`);
    }

    if (onOutput) onOutput(`\r\n=== APPLYING BUNDLED DAWN MOD PAYLOAD ===\r\nTarget: ${installRoot}\r\n`);
    if (onProgress) onProgress(96, 'Deploying Dawn runtime & hooks...');

    return new Promise((resolve, reject) => {
      const child = spawn(
        'powershell.exe',
        ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', psScript, '-GameRoot', installRoot],
        { cwd: bundledDir, windowsHide: true }
      );

      child.stdout.on('data', (d) => {
        if (onOutput) onOutput(d.toString('utf8'));
      });
      child.stderr.on('data', (d) => {
        if (onOutput) onOutput(d.toString('utf8'));
      });

      child.on('close', (code) => {
        if (code === 0) {
          if (onProgress) onProgress(100, 'Dawn successfully installed!');
          if (onOutput) onOutput('\r\n[SUCCESS] Dawn installation complete! Ready to launch.\r\n');
          resolve({ success: true });
        } else {
          reject(new Error(`Install-Dawn.ps1 failed with exit code ${code}`));
        }
      });

      child.on('error', reject);
    });
  }
}

module.exports = new DepotService();
