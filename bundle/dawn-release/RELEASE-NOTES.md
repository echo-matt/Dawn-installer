# Dawn 0.1.3

Performance improvements, mission fixes, and settings that stay saved.

## Fixes and improvements

- FPS stuttering fixes.
- 3D V-Cache support.
- Loadout and Sundial fixes.
- Settings persistence: FOV, primary and secondary key bindings, audio, and supported display preferences now survive a restart.
- Collections item-pulling fixes.
- Omega Baboon error fixes.
- Fixes across all missions, including Deep Storage and Hijacked.
- New Light black-screen fix.
- Screen-size and display-mode fixes.
- New Light door fix. We killed the door.
- Startup crash fix: the release DLL was rebuilt from scratch and checked against the sign-in regression.

## Updating an existing installation

Extract the entire ZIP, close Destiny 2, and run **Update-Dawn.cmd**.
Choose the game folder containing `destiny2.exe`.

The included **Update-Dawn.ps1** preserves existing Dawn saves, inventory, identity, settings, and custom files while updating the DLL and packaged content. It creates a full backup and supports preview (`-WhatIf`) and rollback (`-Restore`). Older Dawn databases upgrade automatically on first launch.

**Install-Dawn.cmd starts a fresh save. Use it only when you want a fresh installation or reset.**

## Requirements and checks

Requires Destiny 2 build **86657.20.08.23.1800.d2_rc** and Windows PowerShell **5.1 or newer**. The game is not included.

The release uses the DLL confirmed working in game. Installer and updater checks passed on PowerShell 5.1 and 7, including an upgrade from 0.1.2, preservation of an older save, database migration, and rollback. Read `READ-ME.txt` for details.
