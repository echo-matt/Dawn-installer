import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';
import { patchAppImageWayland } from './patch-appimage-wayland.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

function bumpVersion(current) {
  const parts = current.split('.').map(Number);
  if (parts.length === 3 && !parts.some(isNaN)) {
    parts[2] += 1;
    return parts.join('.');
  }
  return `${current}.1`;
}

function run() {
  const pkgPath = path.join(rootDir, 'package.json');
  const cargoPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');
  const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');

  // 1. Read current version
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  const oldVersion = pkg.version;

  // Check if custom version passed via CLI: node scripts/release.js 1.1.0
  // Or check if running in GitHub Actions with a tag (v1.0.1) or CI
  const tagMatch = (process.env.GITHUB_REF_NAME || process.env.GITHUB_REF || '').match(/v?(\d+\.\d+\.\d+.*)/);
  const tagVersion = tagMatch ? tagMatch[1] : null;
  const envVersion = tagVersion || (process.env.CI ? oldVersion : null);
  const argVersion = process.argv[2] || envVersion;
  const newVersion = argVersion && /^\d+\.\d+\.\d+/.test(argVersion)
    ? argVersion
    : bumpVersion(oldVersion);

  console.log(`\n========================================`);
  console.log(`  Bumping Dawn Launcher Version`);
  console.log(`  v${oldVersion}  -->  v${newVersion}`);
  console.log(`========================================\n`);

  // 2. Update package.json
  pkg.version = newVersion;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`[OK] Updated package.json to v${newVersion}`);

  // 3. Update Cargo.toml
  let cargoContent = fs.readFileSync(cargoPath, 'utf8');
  cargoContent = cargoContent.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
  fs.writeFileSync(cargoPath, cargoContent);
  console.log(`[OK] Updated src-tauri/Cargo.toml to v${newVersion}`);

  // 4. Update tauri.conf.json
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = newVersion;
  if (tauriConf.app?.windows?.[0]) {
    tauriConf.app.windows[0].title = `Dawn Installer v${newVersion}`;
  }
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');
  console.log(`[OK] Updated src-tauri/tauri.conf.json to v${newVersion}`);

  // 5. Clean previous bundle outputs to guarantee fresh artifacts
  const bundleDir = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle');
  if (fs.existsSync(bundleDir)) {
    try {
      fs.rmSync(bundleDir, { recursive: true, force: true });
    } catch (_) {}
  }

  // 6. Build Tauri release
  console.log(`\n[BUILD] Running Tauri release build (npm run tauri:build)...`);
  execSync('npm run tauri:build', { cwd: rootDir, stdio: 'inherit' });

  const isWindows = process.platform === 'win32';
  if (!isWindows) {
    try {
      patchAppImageWayland();
    } catch (patchErr) {
      console.error('[WAYLAND-PATCH] Error patching AppImage:', patchErr);
    }
  }

  // 7. Copy artifacts to dist-release
  const distReleaseDir = path.join(rootDir, 'dist-release');
  if (!fs.existsSync(distReleaseDir)) {
    fs.mkdirSync(distReleaseDir, { recursive: true });
  }

  const releaseExeCandidates = [
    path.join(rootDir, 'src-tauri', 'target', 'release', isWindows ? 'DAWN.exe' : 'DAWN'),
    path.join(rootDir, 'src-tauri', 'target', 'release', isWindows ? 'dawn.exe' : 'dawn'),
    path.join(rootDir, 'src-tauri', 'target', 'release', isWindows ? 'dawn-launcher.exe' : 'dawn-launcher'),
    path.join(rootDir, 'src-tauri', 'target', 'release', 'DAWN.exe'),
    path.join(rootDir, 'src-tauri', 'target', 'release', 'DAWN')
  ];
  const releaseExe = releaseExeCandidates.find(p => fs.existsSync(p)) || releaseExeCandidates[0];

  // Find nsis setup exe (Windows)
  const nsisDir = path.join(bundleDir, 'nsis');
  if (fs.existsSync(nsisDir)) {
    const nsisFiles = fs.readdirSync(nsisDir)
      .filter(f => f.endsWith('.exe'))
      .sort((a, b) => fs.statSync(path.join(nsisDir, b)).mtimeMs - fs.statSync(path.join(nsisDir, a)).mtimeMs);
    const targetNsis = nsisFiles.find(f => f.includes(newVersion)) || nsisFiles[0];
    if (targetNsis) {
      const srcSetup = path.join(nsisDir, targetNsis);
      const destSetup = path.join(distReleaseDir, `DAWN-Setup-v${newVersion}.exe`);
      fs.copyFileSync(srcSetup, destSetup);
      console.log(`[COPY] Created ${path.basename(destSetup)} (${(fs.statSync(destSetup).size / 1024 / 1024).toFixed(2)} MB) from ${targetNsis}`);
    }
  }

  // Find msi (Windows)
  const msiDir = path.join(bundleDir, 'msi');
  if (fs.existsSync(msiDir)) {
    const msiFiles = fs.readdirSync(msiDir)
      .filter(f => f.endsWith('.msi'))
      .sort((a, b) => fs.statSync(path.join(msiDir, b)).mtimeMs - fs.statSync(path.join(msiDir, a)).mtimeMs);
    const targetMsi = msiFiles.find(f => f.includes(newVersion)) || msiFiles[0];
    if (targetMsi) {
      const srcMsi = path.join(msiDir, targetMsi);
      const destMsi = path.join(distReleaseDir, `DAWN-v${newVersion}.msi`);
      fs.copyFileSync(srcMsi, destMsi);
      console.log(`[COPY] Created ${path.basename(destMsi)} (${(fs.statSync(destMsi).size / 1024 / 1024).toFixed(2)} MB) from ${targetMsi}`);
    }
  }

  // Find deb (Linux)
  const debDir = path.join(bundleDir, 'deb');
  if (fs.existsSync(debDir)) {
    const debFiles = fs.readdirSync(debDir)
      .filter(f => f.endsWith('.deb'))
      .sort((a, b) => fs.statSync(path.join(debDir, b)).mtimeMs - fs.statSync(path.join(debDir, a)).mtimeMs);
    const targetDeb = debFiles.find(f => f.includes(newVersion)) || debFiles[0];
    if (targetDeb) {
      const srcDeb = path.join(debDir, targetDeb);
      const destDeb = path.join(distReleaseDir, `DAWN_${newVersion}_amd64.deb`);
      fs.copyFileSync(srcDeb, destDeb);
      console.log(`[COPY] Created ${path.basename(destDeb)} (${(fs.statSync(destDeb).size / 1024 / 1024).toFixed(2)} MB) from ${targetDeb}`);
    }
  }

  // Find AppImage (Linux)
  const appimageDir = path.join(bundleDir, 'appimage');
  if (fs.existsSync(appimageDir)) {
    const appimageFiles = fs.readdirSync(appimageDir)
      .filter(f => f.endsWith('.AppImage'))
      .sort((a, b) => fs.statSync(path.join(appimageDir, b)).mtimeMs - fs.statSync(path.join(appimageDir, a)).mtimeMs);
    const targetAppImage = appimageFiles.find(f => f.includes(newVersion)) || appimageFiles[0];
    if (targetAppImage) {
      const srcAppImage = path.join(appimageDir, targetAppImage);
      const destAppImage = path.join(distReleaseDir, `DAWN-v${newVersion}.AppImage`);
      fs.copyFileSync(srcAppImage, destAppImage);
      console.log(`[COPY] Created ${path.basename(destAppImage)} (${(fs.statSync(destAppImage).size / 1024 / 1024).toFixed(2)} MB) from ${targetAppImage}`);
    }
  }

  // Create Portable Zip / Tarball
  if (fs.existsSync(releaseExe)) {
    if (isWindows) {
      const destZip = path.join(distReleaseDir, `DAWN-v${newVersion}-Portable.zip`);
      if (fs.existsSync(destZip)) {
        try { fs.unlinkSync(destZip); } catch (e) {}
      }
      try {
        execSync(`tar.exe -a -c -f "${destZip}" -C "${path.dirname(releaseExe)}" "${path.basename(releaseExe)}"`, { cwd: rootDir, stdio: 'inherit' });
      } catch (err) {
        const psCmd = `Compress-Archive -Force -Path '${releaseExe}' -DestinationPath '${destZip}'`;
        execSync(`pwsh -NoProfile -Command "${psCmd}"`, { cwd: rootDir, stdio: 'inherit' });
      }
      if (fs.existsSync(destZip)) {
        console.log(`[ZIP]  Created ${path.basename(destZip)} (${(fs.statSync(destZip).size / 1024 / 1024).toFixed(2)} MB)`);
      }
    } else {
      const destTar = path.join(distReleaseDir, `DAWN-v${newVersion}-linux-x64.tar.gz`);
      if (fs.existsSync(destTar)) {
        try { fs.unlinkSync(destTar); } catch (e) {}
      }
      const dawnShSrc = path.join(rootDir, 'DAWN.sh');
      const dawnShDest = path.join(path.dirname(releaseExe), 'DAWN.sh');
      if (fs.existsSync(dawnShSrc)) {
        fs.copyFileSync(dawnShSrc, dawnShDest);
        try { fs.chmodSync(dawnShDest, 0o755); } catch (_) {}
        execSync(`tar -czf "${destTar}" -C "${path.dirname(releaseExe)}" "${path.basename(releaseExe)}" "DAWN.sh"`, { cwd: rootDir, stdio: 'inherit' });
      } else {
        execSync(`tar -czf "${destTar}" -C "${path.dirname(releaseExe)}" "${path.basename(releaseExe)}"`, { cwd: rootDir, stdio: 'inherit' });
      }
      if (fs.existsSync(destTar)) {
        console.log(`[TAR]  Created ${path.basename(destTar)} (${(fs.statSync(destTar).size / 1024 / 1024).toFixed(2)} MB)`);
      }
    }
  }

  // 8. Verify no personal info leaked into artifacts
  console.log(`\n[SECURITY] Verifying release artifacts for personal info leaks...`);
  const forbidden = ['Matteo', 'matteo', 'VSCode stuff', 'vscode stuff'];
  const filesToCheck = [releaseExe];
  if (fs.existsSync(distReleaseDir)) {
    for (const file of fs.readdirSync(distReleaseDir)) {
      if (file.includes(newVersion)) {
        filesToCheck.push(path.join(distReleaseDir, file));
      }
    }
  }

  let leakDetected = false;
  for (const filePath of filesToCheck) {
    if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) continue;
    const buf = fs.readFileSync(filePath);
    for (const str of forbidden) {
      const bUtf8 = Buffer.from(str, 'utf8');
      const bUtf16 = Buffer.from(str, 'utf16le');
      if (buf.includes(bUtf8) || buf.includes(bUtf16)) {
        console.error(`[LEAK DETECTED] File ${path.basename(filePath)} contains forbidden token: "${str}"!`);
        leakDetected = true;
      }
    }
  }

  if (leakDetected) {
    throw new Error('Build failed security check: Personal paths/usernames found in release binaries!');
  }
  console.log(`[CLEAN] All v${newVersion} artifacts verified 100% clean of personal info.`);

  console.log(`\n========================================`);
  console.log(`  Dist-Release v${newVersion} Packaged Successfully!`);
  console.log(`  Location: dist-release/`);
  console.log(`========================================\n`);
}

run();
