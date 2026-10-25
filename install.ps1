#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$InstallDir = $env:LOCALAPPDATA
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$repo = 'pepestunaa/music-discord-rpc'
$asset = 'music_discord_rpc.exe'
$target = Join-Path $InstallDir $asset

Write-Host 'Music Discord RPC installer'

# Windows locks running executables, so an update has to stop the old copy first.
Get-Process -Name 'music_discord_rpc' -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300

function Install-FromRelease {
    $url = "https://github.com/$repo/releases/latest/download/$asset"
    Write-Host 'Downloading latest release...'
    Invoke-WebRequest -Uri $url -OutFile $target -UseBasicParsing
}

function Install-FromSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw "No release build is published yet and cargo is not on PATH. Install Rust from https://rustup.rs and run this again."
    }
    $work = Join-Path $env:TEMP ("music-discord-rpc-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $work -Force | Out-Null
    try {
        Write-Host 'Fetching source...'
        $zip = Join-Path $work 'source.zip'
        Invoke-WebRequest -Uri "https://codeload.github.com/$repo/zip/refs/heads/main" -OutFile $zip -UseBasicParsing
        Expand-Archive -Path $zip -DestinationPath $work -Force
        $src = Get-ChildItem -Path $work -Directory | Select-Object -First 1
        Write-Host 'Building (this takes a minute)...'
        Push-Location $src.FullName
        try {
            cargo build --release
        } finally {
            Pop-Location
        }
        Copy-Item -Path (Join-Path $src.FullName "target\release\$asset") -Destination $target -Force
    } finally {
        Remove-Item -Path $work -Recurse -Force -ErrorAction SilentlyContinue
    }
}

$installed = $false
try {
    Remove-Item -Path $target -Force -ErrorAction SilentlyContinue
    Install-FromRelease
    $installed = (Test-Path $target) -and ((Get-Item $target).Length -gt 0)
} catch {
    Write-Host 'No release build available; building from source instead.'
}

if (-not $installed) {
    Install-FromSource
    if (-not (Test-Path $target)) {
        throw "Build finished but $asset was not produced."
    }
}

Write-Host "Installed to $target"
Start-Process -FilePath $target -WorkingDirectory $InstallDir
Write-Host 'Running. It registers itself for Windows startup on launch; see the README to undo that.'
