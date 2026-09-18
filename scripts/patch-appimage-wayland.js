import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

export function patchAppImageWayland() {
  const appimageBundleDir = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'appimage');
  if (!fs.existsSync(appimageBundleDir)) {
    console.log('[WAYLAND-PATCH] No appimage bundle dir found, skipping.');
    return false;
  }

  // Find AppDir
  const appDirs = fs.readdirSync(appimageBundleDir)
    .filter(f => f.endsWith('.AppDir') && fs.statSync(path.join(appimageBundleDir, f)).isDirectory());

  if (appDirs.length === 0) {
    console.log('[WAYLAND-PATCH] No .AppDir found in appimage bundle dir, skipping.');
    return false;
  }

  const appDirPath = path.join(appimageBundleDir, appDirs[0]);
  console.log(`[WAYLAND-PATCH] Patching AppDir at ${appDirPath}...`);

  // 1. Ensure apprun-hooks directory
  const hooksDir = path.join(appDirPath, 'apprun-hooks');
  if (!fs.existsSync(hooksDir)) {
    fs.mkdirSync(hooksDir, { recursive: true });
  }

  // 2. Write wayland-compat.sh hook
  const waylandHookPath = path.join(hooksDir, 'wayland-compat.sh');
  const waylandHookContent = `#!/bin/sh
# Tauri 2 AppImage Wayland EGL and client compatibility hook
# Fixes blank/gray screens and EGL_BAD_PARAMETER on Wayland/Hyprland/Arch/SteamOS
export DESKTOPINTEGRATION="\${DESKTOPINTEGRATION:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="\${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"

if [ -z "\${LD_PRELOAD:-}" ]; then
  for lib in \\
    /usr/lib/libwayland-client.so \\
    /usr/lib64/libwayland-client.so \\
    /usr/lib/x86_64-linux-gnu/libwayland-client.so \\
    /usr/lib/aarch64-linux-gnu/libwayland-client.so \\
    /usr/lib/arm-linux-gnueabihf/libwayland-client.so; do
    if [ -f "$lib" ]; then
      export LD_PRELOAD="$lib"
      break
    fi
  done
fi
`;
  fs.writeFileSync(waylandHookPath, waylandHookContent, { mode: 0o755 });
  console.log('[WAYLAND-PATCH] Injected apprun-hooks/wayland-compat.sh');

  // 3. Wrap AppRun so that all apprun-hooks/*.sh are sourced
  const appRunPath = path.join(appDirPath, 'AppRun');
  const appRunWrappedPath = path.join(appDirPath, 'AppRun.wrapped');

  if (!fs.existsSync(appRunWrappedPath)) {
    if (fs.existsSync(appRunPath)) {
      fs.renameSync(appRunPath, appRunWrappedPath);
      try { fs.chmodSync(appRunWrappedPath, 0o755); } catch (_) {}
    }
  }

  const newAppRunContent = `#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "\${0}")")"

# Source compatibility hooks (GTK plugin, Wayland compat, etc.)
if [ -d "$HERE/apprun-hooks" ]; then
  for hook in "$HERE/apprun-hooks/"*.sh; do
    if [ -f "$hook" ]; then
      . "$hook"
    fi
  done
fi

exec "$HERE/AppRun.wrapped" "$@"
`;
  fs.writeFileSync(appRunPath, newAppRunContent, { mode: 0o755 });
  console.log('[WAYLAND-PATCH] Created wrapper AppRun referencing apprun-hooks');

  // 4. Clean previous unpatched AppImages before repacking
  for (const f of fs.readdirSync(appimageBundleDir)) {
    if (f.endsWith('.AppImage')) {
      try { fs.unlinkSync(path.join(appimageBundleDir, f)); } catch (_) {}
    }
  }

  // 5. Find repack tool (appimagetool or linuxdeploy-plugin-appimage from cache)
  let repackCmd = null;
  const homeDir = process.env.HOME || '/root';
  const cachedAppImagePlugin = path.join(homeDir, '.cache', 'tauri', 'linuxdeploy-plugin-appimage.AppImage');

  try {
    execSync('command -v appimagetool', { stdio: 'ignore' });
    repackCmd = `appimagetool "${appDirPath}"`;
  } catch (_) {
    if (fs.existsSync(cachedAppImagePlugin)) {
      repackCmd = `"${cachedAppImagePlugin}" --appimage-extract-and-run --appdir="${appDirPath}"`;
    }
  }

  if (!repackCmd) {
    console.warn('[WAYLAND-PATCH] Warning: No appimagetool or cached linuxdeploy-plugin-appimage found. AppDir patched, but could not repack AppImage.');
    return true;
  }

  console.log(`[WAYLAND-PATCH] Repacking AppImage with command: ${repackCmd}...`);
  execSync(repackCmd, { cwd: appimageBundleDir, stdio: 'inherit' });

  // Verify repacked AppImage exists
  const repackedFiles = fs.readdirSync(appimageBundleDir).filter(f => f.endsWith('.AppImage'));
  if (repackedFiles.length > 0) {
    console.log(`[WAYLAND-PATCH] Successfully repacked: ${repackedFiles.join(', ')}`);
  }

  return true;
}

// Allow direct CLI execution: node scripts/patch-appimage-wayland.js
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  patchAppImageWayland();
}
