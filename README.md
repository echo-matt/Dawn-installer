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

- **Windows**: Download `DAWN-Setup-v1.1.3.exe` (installer), `DAWN-v1.1.3.msi`, or standalone `DAWN-v1.1.3-Portable.zip`.
- **Steam Deck & Linux**: Download `DAWN-v1.1.0.AppImage`, `DAWN_1.1.0_amd64.deb`, or `DAWN-v1.1.0-linux-x64.tar.gz`.

### Install Instructions

#### 1. Keep the Installer in Its Own Separate Folder
* Keep the Dawn Installer in its own standalone location (such as Downloads or a dedicated installer folder).
* The installer and the game must never share the same folder.

#### 2. Create a New, Empty Game Folder
* Create a brand new, empty folder on your drive for the game itself (for example: `C:\Games\Dawn` or `D:\Destiny2-Dawn`).
* **DO NOT** select your retail Steam Destiny 2 folder.
* **DO NOT** select a folder synced with OneDrive or cloud storage.
* **DO NOT** select the folder where the installer itself is running from.

#### 3. Select the Game Folder in the Installer
* Open the Dawn Installer.
* Click the folder selector button and choose the empty game folder you created in Step 2.

#### 4. Log In and Install
* Click **Install**.
* When prompted, log into your Steam account (scan the QR code using your Steam Mobile app or enter your credentials).
* You do **NOT** need Sunrise or any external tools installed. The installer automatically downloads the compatible version of Destiny 2 and applies Dawn over it.
* Keep the installer open until the download finishes completely.

#### 5. Launching the Game
* Click **Launch Game** inside the installer.
* **Fallback**: If the launch button does not start the game, navigate to your game installation folder in File Explorer and launch `destiny2.exe` directly.

#### 6. Troubleshooting / Clean Reinstall
* If the game does not start, verify you have the Microsoft Visual C++ 2015-2022 x64 Redistributable installed.
* Check Windows Defender / Antivirus Protection History to confirm that `bin\x64\steam_api64.dll` was not blocked.
* If files get corrupted or anything breaks, delete everything inside the game installation folder and run the installer again.

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
