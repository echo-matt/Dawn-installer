#requires -Version 5.1
<#
.SYNOPSIS
Installs a prebuilt Dawn release over Destiny 2 build 86657. No build tools required.
.DESCRIPTION
Run from an extracted release ZIP. Every installation starts a fresh profile using
the release defaults. DLLs and Dawn runtime directories are replaced together,
with a persistent rollback journal. Old saves remain in the backup. The original
Sunrise/Restoration directories are left intact and are never imported.
.EXAMPLE
.\Install-Dawn.ps1 -GameRoot 'C:\Destiny 2 Development'
.EXAMPLE
.\Install-Dawn.ps1 -GameRoot 'C:\Destiny 2 Development' -WhatIf
.EXAMPLE
.\Install-Dawn.ps1 -GameRoot 'C:\Destiny 2 Development' -Restore
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string] $GameRoot,
    [switch] $Restore,
    [string] $BackupPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:Utf8 = New-Object System.Text.UTF8Encoding($false)
$script:DisplayTarget = 'user-display/cvars.xml'
$script:Targets = @('Dawn', 'bin/x64/Dawn', 'steam_api64.dll', 'bin/x64/steam_api64.dll', '.dawn/release.json', $script:DisplayTarget)

function Join-SafePath([string] $Root, [string] $Relative) {
    if ([string]::IsNullOrWhiteSpace($Relative) -or $Relative -match '(^[\\/]|:|(^|[\\/])\.\.?([\\/]|$))') {
        throw "Unsafe relative path: $Relative"
    }
    $prefix = [IO.Path]::GetFullPath($Root).TrimEnd('\', '/') + [IO.Path]::DirectorySeparatorChar
    $path = [IO.Path]::GetFullPath((Join-Path $Root $Relative))
    if (-not $path.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw "Path escapes $Root" }
    return $path
}

function Assert-PlainPath([string] $Path) {
    $probe = [IO.Path]::GetFullPath($Path)
    while ($probe) {
        if (Test-Path -LiteralPath $probe) {
            if ((Get-Item -LiteralPath $probe -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Links and junctions are not supported here: $probe"
            }
        }
        $parent = [IO.Path]::GetDirectoryName($probe.TrimEnd('\', '/'))
        if ($parent -eq $probe) { break }
        $probe = $parent
    }
}

function Get-PlainFiles([string] $Root) {
    Assert-PlainPath $Root
    $pending = New-Object 'System.Collections.Generic.Stack[string]'
    $pending.Push($Root)
    while ($pending.Count) {
        foreach ($item in Get-ChildItem -LiteralPath $pending.Pop() -Force) {
            if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Linked entry: $($item.FullName)" }
            if ($item.PSIsContainer) { $pending.Push($item.FullName) } else { $item }
        }
    }
}

function Write-Json([string] $Path, $Value) {
    Assert-PlainPath $Path
    $text = ConvertTo-Json -InputObject $Value -Depth 100 -Compress
    $temporary = $Path + '.writing'
    Assert-PlainPath $temporary
    [IO.File]::WriteAllText($temporary, $text, $script:Utf8)
    if (Test-Path -LiteralPath $Path) { [IO.File]::Replace($temporary, $Path, [NullString]::Value) }
    else { [IO.File]::Move($temporary, $Path) }
}

function Read-Object([string] $Path) {
    Assert-PlainPath $Path
    $value = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    if ($null -eq $value -or $value -isnot [pscustomobject]) { throw "Expected a JSON object: $Path" }
    return $value
}

function Assert-GameClosed {
    if (Get-Process -Name destiny2 -ErrorAction SilentlyContinue) {
        throw 'Close Destiny 2 before installing or restoring. No game process will be stopped automatically.'
    }
}

function Copy-Verified([string] $Source, [string] $Destination) {
    Assert-PlainPath $Source
    Assert-PlainPath $Destination
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Destination)) | Out-Null
    $hash = (Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
    if ((Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash -ne $hash) {
        throw "Copy verification failed: $Destination"
    }
}

function Assert-DawnDll([string] $Path) {
    $stream = [IO.File]::OpenRead($Path)
    $reader = New-Object IO.BinaryReader($stream)
    try {
        if ($stream.Length -lt 256 -or $reader.ReadUInt16() -ne 0x5A4D) { throw 'Release DLL is not a PE image.' }
        $stream.Position = 0x3C
        $offset = $reader.ReadInt32()
        if ($offset -lt 64 -or $offset + 24 -gt $stream.Length) { throw 'Invalid PE header.' }
        $stream.Position = $offset
        if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) { throw 'Release DLL must be Windows x64.' }
        $stream.Position = $offset + 22
        if (-not ($reader.ReadUInt16() -band 0x2000)) { throw 'Release image is not a DLL.' }
    } finally { $reader.Dispose() }
    $version = (Get-Item -LiteralPath $Path).VersionInfo
    if ($version.ProductName -ne 'Dawn' -or $version.IsDebug) {
        throw 'Expected a Dawn Release DLL. Sunrise/Restoration and Debug binaries cannot use this package.'
    }
}

function Read-Release([string] $PackageRoot) {
    $manifest = Read-Object (Join-Path $PackageRoot 'release.json')
    if ($manifest.schema -ne 1 -or $manifest.gameBuild -ne 86657 -or $manifest.runtimeDirectory -cne 'Dawn' -or
        $manifest.release -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,79}$') { throw 'Unsupported release manifest.' }
    $payload = Join-Path $PackageRoot 'payload'
    $known = @{}
    foreach ($entry in $manifest.files) {
        $path = [string]$entry.path
        $allowed = $path -cmatch '^(steam_api64\.dll|Dawn/(settings|hud|movement|player)\.json|Dawn/scripts/[A-Za-z0-9_-]+\.(lua|json)|Dawn/vendor_(catalog|bounty_roll|exchange|item_substitute)\.txt|Dawn/event_presets/[A-Za-z0-9_-]+\.txt|Dawn/licenses/[A-Za-z0-9_.-]+)$'
        if (-not $allowed -or $known.ContainsKey($path) -or $entry.sha256 -notmatch '^[0-9a-fA-F]{64}$') {
            throw "Invalid or duplicate release entry: $path"
        }
        $file = Join-SafePath $payload $path
        Assert-PlainPath $file
        if (-not (Test-Path -LiteralPath $file -PathType Leaf) -or
            (Get-Item -LiteralPath $file).Length -ne $entry.size -or
            (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $entry.sha256) {
            throw "Release file is missing or changed: $path. Extract a fresh release ZIP."
        }
        $known[$path] = $true
    }
    foreach ($required in @('steam_api64.dll', 'Dawn/settings.json', 'Dawn/hud.json', 'Dawn/movement.json',
            'Dawn/player.json', 'Dawn/vendor_catalog.txt', 'Dawn/vendor_bounty_roll.txt',
            'Dawn/vendor_exchange.txt', 'Dawn/vendor_item_substitute.txt')) {
        if (-not $known.ContainsKey($required)) { throw "Incomplete release: $required" }
    }
    if (-not @($known.Keys | Where-Object { $_ -like 'Dawn/scripts/*' }).Count -or
        -not @($known.Keys | Where-Object { $_ -like 'Dawn/event_presets/*' }).Count) { throw 'Release content is missing.' }
    $files = @(Get-PlainFiles $payload)
    if ($files.Count -ne $known.Count) { throw 'The payload contains files not listed in release.json.' }
    foreach ($file in $files) {
        $relative = $file.FullName.Substring($payload.Length + 1).Replace('\', '/')
        if (-not $known.ContainsKey($relative)) { throw "Unlisted payload file: $relative" }
    }
    Assert-DawnDll (Join-Path $payload 'steam_api64.dll')
    return $manifest
}

function Move-Checked([string] $From, [string] $To) {
    Assert-PlainPath $From
    Assert-PlainPath $To
    if (Test-Path -LiteralPath $To) { throw "Refusing to overwrite an existing backup: $To" }
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($To)) | Out-Null
    Move-Item -LiteralPath $From -Destination $To
}

function Get-DisplayPreferencesPath {
    if ([string]::IsNullOrWhiteSpace($env:APPDATA)) { throw "Cannot locate this Windows user's display preferences: APPDATA is missing." }
    $path = Join-SafePath $env:APPDATA 'Bungie/DestinyPC/prefs/cvars.xml'
    Assert-PlainPath $path
    return $path
}

function Get-InstallTarget([string] $Root, [string] $Relative) {
    if ($Relative -ceq $script:DisplayTarget) { return Get-DisplayPreferencesPath }
    return Join-SafePath $Root $Relative
}

# Prepare without writing so malformed preferences and -WhatIf never change the install.
function Get-DisplayDefaults {
    $path = Get-DisplayPreferencesPath
    $exists = Test-Path -LiteralPath $path
    $hash = $null
    $document = New-Object Xml.XmlDocument
    $document.PreserveWhitespace = $true
    $document.XmlResolver = $null
    if ($exists) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or (Get-Item -LiteralPath $path).Length -gt 1MB) {
            throw "Invalid or oversized display preferences: $path"
        }
        $hash = (Get-FileHash -LiteralPath $path).Hash
        $options = New-Object Xml.XmlReaderSettings
        $options.DtdProcessing = [Xml.DtdProcessing]::Prohibit
        $options.XmlResolver = $null
        $reader = [Xml.XmlReader]::Create($path, $options)
        try { $document.Load($reader) } finally { $reader.Dispose() }
    } else {
        $document.LoadXml('<?xml version="1.0"?><body><namespace name="graphics" /></body>')
    }
    if ($null -eq $document.DocumentElement -or $document.DocumentElement.Name -cne 'body' -or
        $document.DocumentElement.NamespaceURI -ne '') { throw "Unrecognized display preferences: $path" }
    $graphics = @($document.SelectNodes('/body/namespace[@name="graphics"]'))
    if ($graphics.Count -gt 1) { throw "Duplicate graphics settings in $path" }
    if ($graphics.Count -eq 0) {
        $section = $document.CreateElement('namespace')
        $section.SetAttribute('name', 'graphics')
        $null = $document.DocumentElement.AppendChild($section)
    } else { $section = $graphics[0] }
    $modes = @($section.SelectNodes('cvar[@name="window_mode"]'))
    if ($modes.Count -gt 1) { throw "Duplicate window_mode settings in $path" }
    if ($modes.Count -eq 0) {
        $mode = $document.CreateElement('cvar')
        $mode.SetAttribute('name', 'window_mode')
        $null = $section.AppendChild($mode)
    } else { $mode = $modes[0] }
    # Native mode 2 follows the desktop. Never distribute one developer's resolution.
    $mode.SetAttribute('value', '2')
    if ($document.FirstChild -is [Xml.XmlDeclaration]) { $document.FirstChild.Encoding = 'utf-8' }
    return [pscustomobject]@{ Path = $path; Existed = $exists; Hash = $hash; Text = $document.OuterXml }
}

function Restore-Transaction([string] $Root, [string] $Transaction) {
    Assert-GameClosed
    $journalPath = Join-SafePath $Transaction 'journal.json'
    $journal = Read-Object $journalPath
    if ($journal.schema -ne 1 -or $journal.gameRoot -ne $Root -or $journal.state -eq 'restored') {
        throw 'This backup belongs to another installation or has already been restored.'
    }
    # Older release backups have no per-user display operation and remain restorable.
    $hasDisplay = $null -ne $journal.PSObject.Properties['displayPreferencesPath']
    $expectedTargets = @($script:Targets | Where-Object { $hasDisplay -or $_ -cne $script:DisplayTarget })
    if ($hasDisplay -and $journal.displayPreferencesPath -ne (Get-DisplayPreferencesPath)) {
        throw 'Restore this backup from the same Windows user profile that installed it.'
    }
    if (@($journal.operations).Count -ne $expectedTargets.Count) { throw 'Invalid backup operation count.' }
    $seen = @{}
    foreach ($operation in $journal.operations) {
        if ($operation.target -cnotin $expectedTargets -or $seen.ContainsKey($operation.target) -or
            $operation.phase -notin @('pending', 'saving', 'saved', 'placing', 'placed', 'restoring', 'restored')) {
            throw 'Invalid backup journal.'
        }
        $seen[$operation.target] = $true
        $target = Get-InstallTarget $Root $operation.target
        Assert-PlainPath $target
        if (Test-Path -LiteralPath $target -PathType Container) { @(Get-PlainFiles $target) | Out-Null }
        $saved = Join-SafePath $Transaction ('previous/' + $operation.target)
        Assert-PlainPath $saved
        if (Test-Path -LiteralPath $saved -PathType Container) { @(Get-PlainFiles $saved) | Out-Null }
    }
    $journal.state = 'restoring'
    Write-Json $journalPath $journal
    $operations = @($journal.operations)
    [array]::Reverse($operations)
    foreach ($operation in $operations) {
        Assert-GameClosed
        if ($operation.phase -eq 'restored') { continue }
        $target = Get-InstallTarget $Root $operation.target
        $saved = Join-SafePath $Transaction ('previous/' + $operation.target)
        $hasBackup = Test-Path -LiteralPath $saved
        # pending/saving without a saved copy means the original never moved. A restoring
        # item with no saved copy has already been returned after a process interruption.
        if (-not $hasBackup -and $operation.existed) {
            if ($operation.phase -notin @('pending', 'saving', 'restoring') -or -not (Test-Path -LiteralPath $target)) {
                throw "Original backup is missing for $($operation.target). Keep $Transaction for recovery."
            }
        } elseif ($hasBackup -or $operation.phase -in @('placing', 'placed', 'restoring')) {
            $operation.phase = 'restoring'
            Write-Json $journalPath $journal
            if (Test-Path -LiteralPath $target) {
                # Preserve progress created since installation, even when the user rolls back.
                $retained = Join-SafePath $Transaction ('after-restore/' + [guid]::NewGuid().ToString('N') + '/' + $operation.target)
                Move-Checked $target $retained
            }
            if ($hasBackup) { Move-Checked $saved $target }
        }
        $operation.phase = 'restored'
        Write-Json $journalPath $journal
    }
    $journal.state = 'restored'
    Write-Json $journalPath $journal
}

function Invoke-DawnInstall {
    if (-not $GameRoot) { $GameRoot = (Read-Host 'Existing game folder (the folder containing destiny2.exe)').Trim('"') }
    $root = [IO.Path]::GetFullPath($GameRoot).TrimEnd('\', '/')
    Assert-PlainPath $root
    $exe = Join-SafePath $root 'destiny2.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw "No destiny2.exe in $root" }
    $version = (Get-Item -LiteralPath $exe).VersionInfo.FileVersion
    if ($version -ne '86657.20.08.23.1800.d2_rc') { throw "Unsupported Destiny build: $version. This installer requires 86657.20.08.23.1800.d2_rc." }
    Assert-GameClosed
    $backupRoot = Join-SafePath $root '.dawn/release-backups'
    Assert-PlainPath $backupRoot
    $history = @()
    if (Test-Path -LiteralPath $backupRoot) {
        $history = @(Get-ChildItem -LiteralPath $backupRoot -Directory | Sort-Object Name -Descending | ForEach-Object {
            Assert-PlainPath $_.FullName
            $path = Join-Path $_.FullName 'journal.json'
            if (Test-Path -LiteralPath $path) {
                [pscustomobject]@{ Path = $_.FullName; Journal = (Read-Object $path); Hash = (Get-FileHash -LiteralPath $path).Hash }
            }
        })
    }
    if ($Restore) {
        $latest = @($history | Where-Object { $_.Journal.state -ne 'restored' } | Select-Object -First 1)
        $selected = if ($BackupPath) { [IO.Path]::GetFullPath($BackupPath).TrimEnd('\', '/') }
                    elseif ($latest.Count) { $latest[0].Path } else { $null }
        if (-not $selected -or [IO.Path]::GetDirectoryName($selected) -ne $backupRoot) { throw 'No matching release backup. -BackupPath must be a direct child of .dawn/release-backups in this game folder.' }
        Assert-PlainPath $selected
        if (-not $PSCmdlet.ShouldProcess($root, "Restore backup $selected (current Dawn files will also be retained)")) { return }
    } else {
        $unfinished = @($history | Where-Object { $_.Journal.state -notin @('complete', 'restored') })
        if ($unfinished.Count) { throw "An interrupted installation needs recovery. Run with -Restore -BackupPath `"$($unfinished[0].Path)`" first." }
        $release = Read-Release $PSScriptRoot
        # Neither the payload nor a source checkout may be inside a directory we replace.
        foreach ($relative in $script:Targets) {
            $target = Get-InstallTarget $root $relative
            Assert-PlainPath $target
            if ($PSScriptRoot -eq $target -or $PSScriptRoot.StartsWith($target + '\', [StringComparison]::OrdinalIgnoreCase)) {
                throw 'Extract the installer outside the Dawn runtime folders.'
            }
            if (Test-Path -LiteralPath $target -PathType Container) {
                if ((Test-Path -LiteralPath (Join-Path $target 'src')) -or (Test-Path -LiteralPath (Join-Path $target 'Dawn.vcxproj'))) {
                    throw "A source checkout occupies $target. Use a separate game installation."
                }
                @(Get-PlainFiles $target) | Out-Null
            }
        }
        foreach ($name in @('settings.json', 'hud.json', 'movement.json', 'player.json')) {
            $defaultPath = Join-Path $PSScriptRoot "payload/Dawn/$name"
            $null = Read-Object $defaultPath
            if ((Get-Item -LiteralPath $defaultPath).Length -ge 1MB) { throw "Release configuration exceeds the runtime limit: $name" }
        }
        $displayDefaults = Get-DisplayDefaults
        Write-Host "Release: $($release.release)"
        Write-Host "Game:    $root"
        Write-Host 'Save:    Fresh profile and release-default settings. Existing Dawn saves will be backed up.'
        Write-Host 'Display: Windowed fullscreen for this Windows user; resolution and other game preferences are preserved.'
        if (-not $PSCmdlet.ShouldProcess($root, 'Back up and replace both Dawn runtime trees and DLL copies; start a fresh save and set windowed fullscreen')) { return }
    }

    $stateDir = Join-SafePath $root '.dawn'
    Assert-PlainPath $stateDir
    [IO.Directory]::CreateDirectory($stateDir) | Out-Null
    $lockPath = Join-SafePath $stateDir 'release-install.lock'
    Assert-PlainPath $lockPath
    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    try {
        Assert-GameClosed
        if ($Restore) {
            Restore-Transaction $root $selected
            Write-Host "Restored. Files displaced by rollback are retained in $selected\after-restore"
            return
        }
        # Another installer may have finished after preflight. Verify the history again under the lock.
        $currentJournals = @(if (Test-Path -LiteralPath $backupRoot) {
            Get-ChildItem -LiteralPath $backupRoot -Directory | Where-Object {
                Test-Path -LiteralPath (Join-Path $_.FullName 'journal.json')
            }
        })
        if ($currentJournals.Count -ne $history.Count) { throw 'Install history changed during preparation. Re-run the installer.' }
        foreach ($record in $history) {
            if ((Get-FileHash -LiteralPath (Join-Path $record.Path 'journal.json')).Hash -ne $record.Hash) {
                throw 'Install history changed during preparation. Re-run the installer.'
            }
        }
        $transaction = Join-SafePath $backupRoot ((Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
        [IO.Directory]::CreateDirectory($transaction) | Out-Null
        $stage = Join-SafePath $transaction 'new'
        [IO.Directory]::CreateDirectory($stage) | Out-Null
        # Stage and hash everything before touching any installed file.
        foreach ($entry in $release.files) {
            $from = Join-SafePath (Join-Path $PSScriptRoot 'payload') $entry.path
            $to = Join-SafePath $stage $entry.path
            Copy-Verified $from $to
            if ((Get-FileHash -LiteralPath $to).Hash -ne $entry.sha256) { throw "Payload changed while staging: $($entry.path)" }
        }
        foreach ($file in @(Get-PlainFiles (Join-Path $stage 'Dawn'))) {
            $relative = $file.FullName.Substring((Join-Path $stage 'Dawn').Length + 1)
            Copy-Verified $file.FullName (Join-SafePath $stage ('bin/x64/Dawn/' + $relative))
        }
        Copy-Verified (Join-Path $stage 'steam_api64.dll') (Join-SafePath $stage 'bin/x64/steam_api64.dll')
        [IO.Directory]::CreateDirectory((Join-Path $stage '.dawn')) | Out-Null
        Write-Json (Join-Path $stage '.dawn/release.json') ([ordered]@{
            schema = 1; release = $release.release; installedUtc = [DateTime]::UtcNow.ToString('o');
            profileMode = 'fresh'; backup = $transaction; files = $release.files
        })
        $displayStage = Join-SafePath $stage $script:DisplayTarget
        [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($displayStage)) | Out-Null
        [IO.File]::WriteAllText($displayStage, $displayDefaults.Text, $script:Utf8)
        $operations = @($script:Targets | ForEach-Object {
            [pscustomobject]@{ target = $_; existed = (Test-Path -LiteralPath (Get-InstallTarget $root $_)); phase = 'pending' }
        })
        $journal = [pscustomobject]@{ schema = 1; gameRoot = $root; release = $release.release; state = 'prepared';
            displayPreferencesPath = $displayDefaults.Path; operations = $operations }
        $journalPath = Join-Path $transaction 'journal.json'
        Write-Json $journalPath $journal
        try {
            $journal.state = 'installing'
            Write-Json $journalPath $journal
            foreach ($operation in $journal.operations) {
                Assert-GameClosed
                $target = Get-InstallTarget $root $operation.target
                if ($operation.target -ceq $script:DisplayTarget) {
                    $displayExists = Test-Path -LiteralPath $target
                    if ($displayExists -ne $displayDefaults.Existed -or
                        ($displayExists -and (Get-FileHash -LiteralPath $target).Hash -ne $displayDefaults.Hash)) {
                        throw 'Display preferences changed during installation. Close the game and retry.'
                    }
                }
                $saved = Join-SafePath $transaction ('previous/' + $operation.target)
                $operation.phase = 'saving'
                Write-Json $journalPath $journal
                if ($operation.existed) { Move-Checked $target $saved }
                $operation.phase = 'saved'
                Write-Json $journalPath $journal
                $operation.phase = 'placing'
                Write-Json $journalPath $journal
                Move-Checked (Join-SafePath $stage $operation.target) $target
                $operation.phase = 'placed'
                Write-Json $journalPath $journal
            }
            foreach ($entry in $release.files) {
                foreach ($prefix in @('', 'bin/x64/')) {
                    if ((Get-FileHash -LiteralPath (Join-SafePath $root ($prefix + $entry.path))).Hash -ne $entry.sha256) {
                        throw "Installed release verification failed: $($entry.path)"
                    }
                }
            }
            $journal.state = 'complete'
            Write-Json $journalPath $journal
        } catch {
            $failure = $_.Exception.Message
            try { Restore-Transaction $root $transaction }
            catch { throw "Installation failed: $failure. Automatic rollback could not finish: $($_.Exception.Message). Close the game and re-run with -Restore -BackupPath `"$transaction`"." }
            throw "Installation failed; previous files were restored. $failure Backup: $transaction"
        }
        Write-Host "Installed Dawn $($release.release)."
        Write-Host 'A fresh save will be created from the release defaults on first launch.'
        Write-Host 'Windowed fullscreen is now the default. Players can change it later in Video settings.'
        Write-Host "Backup: $transaction"
        Write-Host 'Launch destiny2.exe normally. First launch rebuilds caches and may take longer.'
    } finally { $lock.Dispose() }
}

Invoke-DawnInstall
