#requires -Version 5.1
<#
.SYNOPSIS
Updates an existing Dawn installation while keeping its saves and preferences.
.DESCRIPTION
Run from the extracted release ZIP with Install-Dawn.ps1, release.json, and payload
beside this script. Destiny 2 must be closed. A complete backup supports rollback.
.EXAMPLE
.\Update-Dawn.ps1 -GameRoot 'D:\Games\Destiny 2'
.EXAMPLE
.\Update-Dawn.ps1 -GameRoot 'D:\Games\Destiny 2' -WhatIf
.EXAMPLE
.\Update-Dawn.ps1 -GameRoot 'D:\Games\Destiny 2' -Restore
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string] $GameRoot,
    [switch] $Restore,
    [string] $BackupPath
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$arguments = @{}
foreach ($key in $PSBoundParameters.Keys) { $arguments[$key] = $PSBoundParameters[$key] }
if (-not $Restore) { $arguments.Update = $true }
& (Join-Path $PSScriptRoot 'Install-Dawn.ps1') @arguments
