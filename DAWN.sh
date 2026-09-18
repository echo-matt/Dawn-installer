#!/usr/bin/env sh
# DAWN Launcher - Linux Startup Wrapper
# Resolves WebKitGTK DMABUF Wayland crashes and handles missing libraries via steam-run on SteamOS/NixOS.

set -e

# Disable WebKitGTK hardware DMABUF surface acceleration on Wayland/Mesa/AMD/NVIDIA
# (Prevents WebKitWebProcess Signal 6 ABRT core dumps on SteamOS, Arch, and Wayland)
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"

HERE="$(cd "$(dirname "$0")" && pwd)"
BIN="$HERE/DAWN"

if [ ! -f "$BIN" ]; then
    echo "[ERROR] DAWN binary not found at $BIN"
    exit 1
fi

chmod +x "$BIN" 2>/dev/null || true

# Check if libwebkit2gtk-4.1 is missing and steam-run is available (e.g. SteamOS Desktop / NixOS)
if command -v steam-run >/dev/null 2>&1; then
    # Test if libwebkit2gtk-4.1 is present on host system
    if command -v ldconfig >/dev/null 2>&1; then
        if ! ldconfig -p 2>/dev/null | grep -q "libwebkit2gtk-4.1"; then
            echo "[DAWN] Host system lacks libwebkit2gtk-4.1; launching via steam-run..."
            exec steam-run "$BIN" "$@"
        fi
    elif [ ! -f "/usr/lib/libwebkit2gtk-4.1.so.0" ] && [ ! -f "/usr/lib64/libwebkit2gtk-4.1.so.0" ] && [ ! -f "/usr/lib/x86_64-linux-gnu/libwebkit2gtk-4.1.so.0" ]; then
        echo "[DAWN] libwebkit2gtk-4.1 not found in standard paths; launching via steam-run..."
        exec steam-run "$BIN" "$@"
    fi
fi

exec "$BIN" "$@"