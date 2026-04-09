#Requires -Version 5.1
<#
.SYNOPSIS
    GeliShell installer for Windows (PowerShell)

.DESCRIPTION
    Installs GeliShell from a local clone:

      1. geli.exe + gerisabet.exe  ->  %USERPROFILE%\.local\bin\
      2. docs.db (if found in assets/)  ->  %USERPROFILE%\.config\geliShell\docs\docs.db

    sqlite-vec (vec0.dll) and docs.db are downloaded automatically by GeliShell
    on first run if they are not already present (see bootstrap.rs).
    This installer only seeds docs.db from the repo when available.

.EXAMPLE
    .\install.ps1                     # interactive install
    .\install.ps1 -Force              # overwrite all existing files
    .\install.ps1 -SkipDocs           # skip docs.db seeding from assets\
    .\install.ps1 -BinDir "C:\bin"    # custom binary directory
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$SkipDocs,
    [string]$BinDir = ''
)

$ErrorActionPreference = 'Stop'

# ── Load shared library ───────────────────────────────────────
. (Join-Path $PSScriptRoot 'installer\lib\common.ps1')

# ══════════════════════════════════════════════════════════════
# MAIN — wrapped in try/catch so Invoke-Rollback always runs on failure
# ══════════════════════════════════════════════════════════════
try
{
    # ── Banner ────────────────────────────────────────────────
    Write-Host ""
    Write-Host "  GeliShell Installer  |  Windows PowerShell" -ForegroundColor Magenta
    Write-Host ""

    # ── Project root ──────────────────────────────────────────
    $ProjectRoot = $PSScriptRoot
    if (-not (Test-Path (Join-Path $ProjectRoot 'Cargo.toml')))
    {
        Write-Fail "Run this script from the GeliShell project root (where Cargo.toml lives)"
    }
    Write-Info "project root: $ProjectRoot"

    # ── Pre-flight: require pre-compiled binaries ─────────────
    # This installer copies pre-built binaries — it never invokes cargo.
    foreach ($Bin in @('geli.exe', 'gerisabet.exe'))
    {
        $BinSrc = Join-Path $ProjectRoot "target\release\$Bin"
        if (-not (Test-Path $BinSrc))
        {
            Write-Fail "Binary not found: target\release\$Bin`n       Run first: cargo build --release"
        }
    }

    # ── Resolve paths ─────────────────────────────────────────
    $UserProfile = $env:USERPROFILE
    if (-not $UserProfile)
    {
        Write-Fail "USERPROFILE environment variable is not set"
    }

    if ($BinDir -eq '')
    {
        $BinDir = Join-Path $UserProfile '.local\bin'
    }

    $ConfigRoot  = Join-Path $UserProfile  '.config\geliShell'
    $ModelsDir   = Join-Path $ConfigRoot   'models'
    $DocsDir     = Join-Path $ConfigRoot   'docs'
    $DocsDbDest  = Join-Path $DocsDir      'docs.db'

    # ── Create directories (idempotent) ───────────────────────
    foreach ($Dir in @($BinDir, $ConfigRoot, $ModelsDir, $DocsDir))
    {
        if (-not (Test-Path $Dir))
        {
            New-Item -ItemType Directory -Path $Dir -Force | Out-Null
        }
    }

    # ══════════════════════════════════════════════════════════
    # STEP 1 — geli.exe + gerisabet.exe binaries
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Step "installing GeliShell binaries..."

    foreach ($Bin in @('geli.exe', 'gerisabet.exe'))
    {
        $Src  = Join-Path $ProjectRoot "target\release\$Bin"
        $Dest = Join-Path $BinDir $Bin
        Copy-Item -Path $Src -Destination $Dest -Force
        Register-Rollback -Path $Dest
        Write-Ok "$Bin -> $Dest"
    }

    $GeliBin = Join-Path $BinDir 'geli.exe'

    # PATH injection — User scope registry, idempotent
    $CurrentPath = [System.Environment]::GetEnvironmentVariable('PATH', 'User')
    $BinDirNorm  = $BinDir.TrimEnd('\')
    $AlreadyIn   = ($CurrentPath -split ';') |
                       Where-Object { $_.TrimEnd('\') -eq $BinDirNorm }
    if ($AlreadyIn)
    {
        Write-Info "$BinDir already in user PATH"
    }
    else
    {
        $NewPath = if ($CurrentPath) { "$CurrentPath;$BinDirNorm" } else { $BinDirNorm }
        [System.Environment]::SetEnvironmentVariable('PATH', $NewPath, 'User')
        $env:PATH = "$env:PATH;$BinDirNorm"
        Write-Ok "added to user PATH: $BinDirNorm"
        Write-Warn "Restart your terminal for PATH to take effect in new sessions"
    }

    # ══════════════════════════════════════════════════════════
    # STEP 2 — docs.db  (pre-generated release artifact; no cargo)
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Step "seeding docs.db (RAG knowledge base)..."

    $DocsDbOk = $false

    if ((Test-Path $DocsDbDest) -and -not $Force)
    {
        Write-Ok "docs.db already present: $DocsDbDest"
        $DocsDbOk = $true
    }
    elseif ($SkipDocs)
    {
        Write-Info "skipping docs.db seeding (-SkipDocs)"
    }
    else
    {
        $DocsCandidates = @(
            (Join-Path $ProjectRoot 'assets\docs.db'),
            (Join-Path $ProjectRoot 'docs.db'),
            (Join-Path $ProjectRoot 'docs\docs.db')
        )
        foreach ($Candidate in $DocsCandidates)
        {
            if (Test-Path $Candidate)
            {
                Copy-Item -Path $Candidate -Destination $DocsDbDest -Force
                Register-Rollback -Path $DocsDbDest
                Write-Ok "docs.db seeded from: $Candidate"
                $DocsDbOk = $true
                break
            }
        }

        if (-not $DocsDbOk)
        {
            Write-Warn "docs.db not found in release assets."
            Write-Info "The AI assistant will not work until docs.db is distributed."
            Write-Info "Expected location : $ProjectRoot\assets\docs.db"
            Write-Info "Developer build   : cargo run --bin build_docs_db"
        }
    }

    # ══════════════════════════════════════════════════════════
    # STEP 3 — Post-installation verification
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Step "verifying installation..."

    $GeliVersionOk = $false
    try
    {
        $GeliVer = & $GeliBin --version 2>&1 | Select-Object -First 1
        Write-Ok "geli --version: $GeliVer"
        $GeliVersionOk = $true
    }
    catch
    {
        Write-Warn "geli.exe --version failed — binary may need additional system libraries"
        Write-Info "Try running: $GeliBin --version"
    }

    # ══════════════════════════════════════════════════════════
    # SUMMARY
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Host "  ──────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host "  GeliShell Installation Summary"             -ForegroundColor Magenta
    Write-Host "  ──────────────────────────────────────────" -ForegroundColor DarkGray
    Write-Host ""

    Write-StatusLine -Ok $true      -Label 'geli.exe'      -Detail $GeliBin
    Write-StatusLine -Ok $true      -Label 'gerisabet.exe' -Detail (Join-Path $BinDir 'gerisabet.exe')
    Write-StatusLine -Ok $DocsDbOk  -Label 'docs.db'       -Detail $DocsDbDest

    Write-Host ""
    if ($DocsDbOk)
    {
        Write-Host "  All components ready." -ForegroundColor Green
    }
    else
    {
        Write-Host "  GeliShell core is installed and fully functional." -ForegroundColor Green
        Write-Host "  docs.db and sqlite-vec will be downloaded automatically at first run." -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "  Open a new terminal and run: geli" -ForegroundColor Cyan
    Write-Host ""
}
catch
{
    Write-Host " [ERROR] Installation failed: $_" -ForegroundColor Red
    Invoke-Rollback
    exit 1
}
