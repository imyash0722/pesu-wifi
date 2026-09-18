#Requires -Version 5.1
<#
.SYNOPSIS
    PESU WiFi CLI & Watchdog Installer for Windows.
.DESCRIPTION
    Installs pesu-wifi.exe to %LOCALAPPDATA%\Programs\pesu-wifi, configures PATH,
    and provides options for background startup and scheduled tasks.
#>

param(
    [switch]$FromSource,
    [string]$Version = "latest"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " PESU WiFi Windows CLI & Watchdog Setup   " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\pesu-wifi"
$ExePath = Join-Path $InstallDir "pesu-wifi.exe"
$RepoOwner = "imyash0722"
$RepoName = "pesu-wifi"

# Ensure install directory exists
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$Installed = $false

# 1. Install from Source if requested or in repo
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$IsRepo = Test-Path (Join-Path $ScriptDir "Cargo.toml")

if ($FromSource -or ($IsRepo -and (Get-Command cargo -ErrorAction SilentlyContinue))) {
    Write-Host "[1/3] Building from source with Cargo..." -ForegroundColor Yellow
    Push-Location $ScriptDir
    try {
        cargo build --release
        $BuiltBin = Join-Path $ScriptDir "target\release\pesu-wifi.exe"
        if (Test-Path $BuiltBin) {
            Copy-Item -Path $BuiltBin -Destination $ExePath -Force
            $Installed = $true
            Write-Host "  ✔ Successfully built and installed pesu-wifi.exe" -ForegroundColor Green
        }
    } catch {
        Write-Warning "Source build failed: $_"
    } finally {
        Pop-Location
    }
}

# 2. Download precompiled release binary if not built from source
if (-not $Installed) {
    Write-Host "[1/3] Fetching release from GitHub..." -ForegroundColor Yellow
    $ApiUrl = if ($Version -eq "latest") {
        "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"
    } else {
        "https://api.github.com/repos/$RepoOwner/$RepoName/releases/tags/$Version"
    }

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $ReleaseJson = Invoke-RestMethod -Uri $ApiUrl -Headers @{ "User-Agent" = "pesu-wifi-installer" }
        $Asset = $ReleaseJson.assets | Where-Object { $_.name -like "*windows-x86_64.zip" -or $_.name -like "*windows*.zip" } | Select-Object -First 1

        if (-not $Asset) {
            throw "No Windows zip release asset found in release '$($ReleaseJson.tag_name)'."
        }

        $ZipUrl = $Asset.browser_download_url
        $TempZip = Join-Path $env:TEMP "pesu-wifi-windows.zip"
        Write-Host "  Downloading $($Asset.name)..." -ForegroundColor Gray
        Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip

        Write-Host "  Extracting pesu-wifi.exe to $InstallDir..." -ForegroundColor Gray
        Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
        Remove-Item -Path $TempZip -Force -ErrorAction SilentlyContinue
        $Installed = $true
        Write-Host "  ✔ Installed release $($ReleaseJson.tag_name)" -ForegroundColor Green
    } catch {
        Write-Error "Failed to download binary release: $_"
        exit 1
    }
}

# 3. Add to User PATH
Write-Host "[2/3] Configuring User PATH environment variable..." -ForegroundColor Yellow
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$PathEntries = $UserPath -split ";" | Where-Object { $_ -ne "" }
if ($PathEntries -notcontains $InstallDir) {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "  ✔ Added $InstallDir to User PATH" -ForegroundColor Green
} else {
    Write-Host "  ✔ $InstallDir is already in User PATH" -ForegroundColor Green
}

# 4. Final verification and usage info
Write-Host "[3/3] Verifying installation..." -ForegroundColor Yellow
if (Test-Path $ExePath) {
    Write-Host "  ✔ pesu-wifi.exe is ready!" -ForegroundColor Green
} else {
    Write-Error "pesu-wifi.exe was not found in $InstallDir"
    exit 1
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host " Installation Complete!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Quick start instructions:" -ForegroundColor Cyan
Write-Host "  1. Restart your terminal (PowerShell or Windows Terminal) to refresh PATH."
Write-Host "  2. Save your campus Wi-Fi credentials:"
Write-Host "       pesu-wifi add" -ForegroundColor White
Write-Host "  3. Start the background watchdog service:"
Write-Host "       pesu-wifi start" -ForegroundColor White
Write-Host "  4. Check network status:"
Write-Host "       pesu-wifi status" -ForegroundColor White
Write-Host ""
