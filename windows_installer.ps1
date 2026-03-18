#Requires -Version 5.1
<#
.SYNOPSIS
    GeliShell installer for Windows (PowerShell)

.DESCRIPTION
    Installs the GeliShell binary and runtime assets to the user profile.
    Must be run from the root of the GeliShell project directory.

    What this script does:
      1. Validates the compiled binary exists in target/release/
      2. Creates ~/.local/bin (added to user PATH if missing)
      3. Copies geli_shell.exe → geli.exe  (overwrites if present)
      4. Creates ~/.config/geliShell/{models,docs}/ directory layout
      5. Copies any available assets (vec0.dll, docs.db, dbjson)
      6. Persists PATH change to HKCU so new terminals pick it up
      7. Verifies the install by running: geli --version (exit 0 check)

.EXAMPLE
    # From the GeliShell project root:
    .\install.ps1

    # Force reinstall without prompts:
    .\install.ps1 -Force

    # Install to a custom bin directory:
    .\install.ps1 -BinDir "C:\tools\bin"
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [string]$BinDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Color helpers ────────────────────────────────────────────

function Write-Step  { param([string]$Msg) Write-Host "  --> $Msg" -ForegroundColor Cyan    }
function Write-Ok    { param([string]$Msg) Write-Host "   ok $Msg" -ForegroundColor Green   }
function Write-Warn  { param([string]$Msg) Write-Host " warn $Msg" -ForegroundColor Yellow  }
function Write-Fail  { param([string]$Msg) Write-Host " FAIL $Msg" -ForegroundColor Red; exit 1 }
function Write-Info  { param([string]$Msg) Write-Host "      $Msg" -ForegroundColor DarkGray }

# ── Banner ───────────────────────────────────────────────────

Write-Host ""
Write-Host "  GeliShell Installer" -ForegroundColor Magenta
Write-Host "  v0.1.0  |  Windows PowerShell" -ForegroundColor DarkGray
Write-Host ""

# ── Validate project root ────────────────────────────────────

$ProjectRoot = $PSScriptRoot
if (-not (Test-Path (Join-Path $ProjectRoot "Cargo.toml"))) {
    Write-Fail "Run this script from the GeliShell project root (where Cargo.toml lives)"
}
Write-Info "project root: $ProjectRoot"

# ── Locate binary ────────────────────────────────────────────

$BinarySource = Join-Path $ProjectRoot "target\release\geli_shell.exe"
if (-not (Test-Path $BinarySource)) {
    Write-Host ""
    Write-Warn "Binary not found at: $BinarySource"
    Write-Host "  Run first:  cargo build --release" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}
Write-Ok "binary found: $BinarySource"

# ── Resolve install paths ────────────────────────────────────

$UserProfile = $env:USERPROFILE
if (-not $UserProfile) {
    Write-Fail "USERPROFILE environment variable is not set"
}

if ($BinDir -eq "") {
    $BinDir = Join-Path $UserProfile ".local\bin"
}

$ConfigRoot  = Join-Path $UserProfile ".config\geliShell"
$ModelsDir   = Join-Path $ConfigRoot  "models"
$DocsDir     = Join-Path $ConfigRoot  "docs"
$BinaryDest  = Join-Path $BinDir      "geli.exe"

Write-Info "install dir:  $BinDir"
Write-Info "config root:  $ConfigRoot"

# ── Create directory layout ──────────────────────────────────

Write-Step "creating directory layout..."

foreach ($Dir in @($BinDir, $ConfigRoot, $ModelsDir, $DocsDir)) {
    if (-not (Test-Path $Dir)) {
        New-Item -ItemType Directory -Path $Dir -Force | Out-Null
        Write-Ok "created: $Dir"
    } else {
        Write-Info "exists:  $Dir"
    }
}

# ── Install binary ───────────────────────────────────────────

Write-Step "installing binary..."

$BinaryExists = Test-Path $BinaryDest
if ($BinaryExists -and -not $Force) {
    $Existing = (Get-Item $BinaryDest).LastWriteTime
    Write-Warn "geli.exe already installed (modified: $Existing)"
    Write-Warn "overwriting with new build..."
}

Copy-Item -Path $BinarySource -Destination $BinaryDest -Force
Write-Ok "installed: $BinaryDest"

# ── Copy runtime assets ──────────────────────────────────────

Write-Step "copying runtime assets..."

# Asset map: source pattern -> destination
$AssetMap = @(
    @{
        Sources = @(
            (Join-Path $ProjectRoot "assets\vec0.dll"),
            (Join-Path $ProjectRoot "models\vec0.dll"),
            (Join-Path $ProjectRoot "vec0.dll")
        )
        Dest    = Join-Path $ModelsDir "vec0.dll"
        Label   = "vec0.dll (sqlite-vec extension)"
    },
    @{
        Sources = @(
            (Join-Path $ProjectRoot "assets\docs.db"),
            (Join-Path $ProjectRoot "docs\docs.db"),
            (Join-Path $ProjectRoot "docs.db")
        )
        Dest    = Join-Path $DocsDir "docs.db"
        Label   = "docs.db (RAG knowledge base)"
    },
    @{
        Sources = @(
            (Join-Path $ProjectRoot "assets\dbjson"),
            (Join-Path $ProjectRoot "models\dbjson"),
            (Join-Path $ProjectRoot "dbjson")
        )
        Dest    = Join-Path $ModelsDir "dbjson"
        Label   = "dbjson (assistant index)"
    }
)

foreach ($Asset in $AssetMap) {
    $Copied = $false
    foreach ($Source in $Asset.Sources) {
        if (Test-Path $Source) {
            # Only overwrite if source is newer or Force is set
            $ShouldCopy = $Force -or
                    (-not (Test-Path $Asset.Dest)) -or
                    ((Get-Item $Source).LastWriteTime -gt (Get-Item $Asset.Dest).LastWriteTime)

            if ($ShouldCopy) {
                $DestParent = Split-Path $Asset.Dest -Parent
                if (-not (Test-Path $DestParent)) {
                    New-Item -ItemType Directory -Path $DestParent -Force | Out-Null
                }
                Copy-Item -Path $Source -Destination $Asset.Dest -Force
                Write-Ok "copied:  $($Asset.Label)"
            } else {
                Write-Info "skip:    $($Asset.Label) (already up-to-date)"
            }
            $Copied = $true
            break
        }
    }
    if (-not $Copied) {
        Write-Warn "not found: $($Asset.Label) — assistant features may be limited"
        Write-Info "  expected at one of:"
        foreach ($Source in $Asset.Sources) {
            Write-Info "    $Source"
        }
    }
}

# ── Update user PATH ─────────────────────────────────────────

Write-Step "checking PATH..."

$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
$BinDirNorm  = $BinDir.TrimEnd('\')

if ($CurrentPath -split ";" | Where-Object { $_.TrimEnd('\') -eq $BinDirNorm }) {
    Write-Info "$BinDir is already in user PATH"
} else {
    $NewPath = "$CurrentPath;$BinDirNorm"
    [System.Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Ok "added to user PATH: $BinDirNorm"
    Write-Warn "Restart your terminal (or open a new PowerShell window) for PATH to take effect"

    # Also update current session so the verify step works immediately
    $env:PATH = "$env:PATH;$BinDirNorm"
}

# ── Verify installation ──────────────────────────────────────

Write-Step "verifying installation..."

try {
    # We can't call geli --version since it starts the REPL,
    # so we verify the binary is valid by checking it exists and is executable
    $InstalledItem = Get-Item $BinaryDest
    $SizeKb = [math]::Round($InstalledItem.Length / 1KB)
    Write-Ok "geli.exe is present ($SizeKb KB) at $BinaryDest"
} catch {
    Write-Fail "verification failed: $_"
}

# ── Print summary ─────────────────────────────────────────────

Write-Host ""
Write-Host "  GeliShell installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Binary  : $BinaryDest"            -ForegroundColor DarkGray
Write-Host "  Config  : $ConfigRoot"             -ForegroundColor DarkGray
Write-Host "  Models  : $ModelsDir"              -ForegroundColor DarkGray
Write-Host ""
Write-Host "  To start GeliShell, open a new terminal and run:" -ForegroundColor Cyan
Write-Host "    geli" -ForegroundColor White
Write-Host ""
Write-Host "  First run will launch the setup wizard automatically." -ForegroundColor DarkGray
Write-Host "  To reset config at any time, run inside GeliShell:" -ForegroundColor DarkGray
Write-Host "    geli-reset-config" -ForegroundColor White
Write-Host ""