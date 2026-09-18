# DAWN installer

A modern, fast, and cross-platform desktop installer and launcher for [**Dawn**](https://github.com/isinternets/Dawn).
---

## Overview

This installer simplifies the entire process of installing, managing, and playing Dawn, it automates game file verification, official depot downloads from Steam, mod deployment, and Proton configuration on Linux and Steam Deck.

---

## Features

- **Cross-Platform Support**: Built natively for Windows 10/11 and Linux / SteamOS (Steam Deck).
- **Automated Game Detection**: Automatically scans standard Steam libraries to locate your Destiny 2 folder and validates build compatibility.
- **Integrated Steam Depot Downloader**:
  - Securely authenticates with Steam via **Steam Mobile QR code** or account credentials.
  - Downloads the required build depots directly from Steam servers with real-time download speed and progress reporting.
- **One-Click Dawn Installation**: Deploys the latest Dawn runtime, proxy libraries, default profiles, vendor configs, and launch scripts in seconds.
- **Automatic Version Updates**: Checks GitHub Releases on launch and lets you update your Dawn mod with a single click while preserving your custom configs.
- **Multi-Language Support**: Choose your preferred in-game audio and text language from 13 supported languages.
- **Native Steam Deck / Linux Gaming**: Automatically resolves Proton and Steam compatibility data paths to run Dawn seamlessly on SteamOS.
- **Clean One-Click Uninstaller**: Safely restores your genuine original `steam_api64.dll` from backup and purges all Dawn mod files and caches when you want to return to vanilla.
- **Diagnostics & Tools**: Built-in cache clearing.

---

## Getting Started

### 1. Download

Head over to the [Releases](https://github.com/echo-matt/Dawn-installer/releases) page to download the latest version for your platform:

- **Windows**: Download `DAWN-v1.0.0-Setup.exe` (installer) or standalone `DAWN-v1.0.0-windows-x64.zip`.
- **Steam Deck & Linux**: Download `DAWN-v1.0.0.AppImage`, `DAWN_1.0.0_amd64.deb`, or `DAWN-v1.0.0-linux-x64.tar.gz`.

### 2. Quick Setup

1. **Select Game Folder**:
   - Launch DAWN installer.
   - Click the directory pill in the bottom-right corner to select your Destiny 2 folder (or let the app auto-detect an existing install).
2. **Select Language**:
   - Use the language selector to pick your preferred game depot language (e.g., English, French, German, Spanish, Japanese).
3. **Install**:
   - Click **INSTALL**.
   - If your folder does not have correct build files yet, DAWN installer will prompt you to authenticate with Steam. Scan the QR code with your Steam Mobile app or enter your credentials. (DAWN installer does not hold any personal data about your steam account)
   - DAWN will download the game files and automatically install the Dawn mod.
4. **Launch**:
   - When finished, click **LAUNCH** to start playing Dawn!

---

## Managing Dawn

Click the arrow next to the main action button to access installer tools:

- **Change Game Directory...**: Switch to a different Destiny 2 installation.
- **Update / Reinstall Dawn Mod**: Re-deploy the latest Dawn release or apply new mod updates.
- **Clear Dawn Cache**: Delete compiled shader and mission caches without touching your settings or save files.
- **Open Game Folder**: Quickly open your Destiny 2 game directory in File Explorer / file manager.
- **View Debug Logs**: Open the integrated live console to view detailed background tasks and diagnostic logs.
- **Uninstall Dawn Mod**: Completely uninstall the Dawn mod, remove mod directories and configs, and restore your genuine `steam_api64.dll`.

---

## System Requirements

- **Operating System**:
  - Windows 10 or Windows 11 (64-bit)
  - Linux (Ubuntu 20.04+, Arch Linux, Fedora, SteamOS 3.0+ / Steam Deck)
- **Steam Account**: A free Steam account that owns Destiny 2 (free-to-play) is required for downloading game depots.
- **Storage**: ~105 GB of available disk space for game assets and mod files.

---

## Credits & Disclaimer

- Dawn is developed and maintained by the [Dawn Project Team](https://x.com/dawndevteam).
- Destiny 2 is a registered trademark of Bungie, Inc. DAWN installer is an unofficial open-source installer client and is not affiliated with or endorsed by Bungie or Sony.
