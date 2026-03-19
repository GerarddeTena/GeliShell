#Requires -Version 5.1
<#
.SYNOPSIS
    GeliShell installer for Windows (PowerShell)

.DESCRIPTION
    Installs GeliShell and all required runtime dependencies:

      1. geli.exe binary  → %USERPROFILE%\.local\bin\geli.exe
      2. sqlite-vec        → %USERPROFILE%\.config\geliShell\models\vec0.dll
      3. docs.db           → generated via: cargo run --bin build_docs_db

    IMPORTANT — sqlite-vec is NOT the same as SQLite:
      SQLite   → standard relational database, likely already on your system
      sqlite-vec → a SEPARATE vector-search C extension by Alex Garcia
                   https://github.com/asg017/sqlite-vec
                   GeliShell downloads vec0.dll from its GitHub releases.

    build_docs_db requires Ollama running with nomic-embed-text:
      https://ollama.com/download
      ollama pull nomic-embed-text

.EXAMPLE
    .\install.ps1                     # interactive install
    .\install.ps1 -Force              # overwrite all existing files
    .\install.ps1 -SkipDocs           # skip docs.db generation
    .\install.ps1 -BinDir "C:\bin"    # custom binary directory
#>

[CmdletBinding()]
param(
    [switch]$Force,
    [switch]$SkipDocs,
    [string]$BinDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Helpers ───────────────────────────────────────────────────

function Write-Step  { param([string]$Msg) Write-Host "  --> $Msg" -ForegroundColor Cyan    }
function Write-Ok    { param([string]$Msg) Write-Host "   ok $Msg" -ForegroundColor Green   }
function Write-Warn  { param([string]$Msg) Write-Host " warn $Msg" -ForegroundColor Yellow  }
function Write-Info  { param([string]$Msg) Write-Host "      $Msg" -ForegroundColor DarkGray }
function Write-Fail  { param([string]$Msg) Write-Host " FAIL $Msg" -ForegroundColor Red; exit 1 }

function Ask-YesNo {
    param([string]$Question, [bool]$Default = $true)
    $hint = if ($Default) { "[Y/n]" } else { "[y/N]" }
    Write-Host "  $Question $hint " -ForegroundColor Cyan -NoNewline
    $answer = (Read-Host).Trim().ToLower()
    if ($answer -eq "") { return $Default }
    return ($answer -eq "y" -or $answer -eq "yes")
}

function Write-StatusLine {
    param([bool]$Ok, [string]$Label, [string]$Detail)
    $icon  = if ($Ok) { "  [OK]" } else { "  [--]" }
    $color = if ($Ok) { "Green" } else { "DarkGray" }
    Write-Host "$icon $Label" -ForegroundColor $color
    if ($Detail) { Write-Host "       $Detail" -ForegroundColor DarkGray }
}

function Invoke-Vec0Download {
    param([string]$DestPath)
    Write-Info "fetching latest release info from GitHub API..."
    try {
        $Headers = @{
            "User-Agent" = "GeliShell-Installer/0.1"
            "Accept"     = "application/vnd.github+json"
        }
        $Release = Invoke-RestMethod `
            -Uri "https://api.github.com/repos/asg017/sqlite-vec/releases/latest" `
            -Headers $Headers `
            -TimeoutSec 15

        $Tag = $Release.tag_name
        Write-Info "latest sqlite-vec release: $Tag"

        $Asset = $Release.assets | Where-Object {
            $_.name -match "loadable-windows-x86_64\.zip$"
        } | Select-Object -First 1

        if (-not $Asset) {
            Write-Warn "Windows x64 loadable zip not found in release $Tag"
            Write-Info "Available assets:"
            $Release.assets | ForEach-Object { Write-Info "  $($_.name)" }
            return $false
        }

        $TempZip = Join-Path $env:TEMP "sqlite-vec-$Tag.zip"
        $TempDir = Join-Path $env:TEMP "sqlite-vec-extract-$Tag"

        Write-Info "downloading: $($Asset.name) ..."
        Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $TempZip -TimeoutSec 120

        if (Test-Path $TempDir) { Remove-Item $TempDir -Recurse -Force }
        Expand-Archive -Path $TempZip -DestinationPath $TempDir -Force

        $Dll = Get-ChildItem -Path $TempDir -Recurse -Filter "vec0.dll" |
                Select-Object -First 1

        if (-not $Dll) {
            $Dll = Get-ChildItem -Path $TempDir -Recurse -Filter "*.dll" |
                    Select-Object -First 1
        }

        if (-not $Dll) {
            Write-Warn "vec0.dll not found inside the downloaded archive."
            return $false
        }

        $Parent = Split-Path $DestPath -Parent
        if (-not (Test-Path $Parent)) {
            New-Item -ItemType Directory -Path $Parent -Force | Out-Null
        }
        Copy-Item -Path $Dll.FullName -Destination $DestPath -Force

        Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
        Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue

        Write-Ok "vec0.dll installed at: $DestPath"
        return $true

    } catch {
        Write-Warn "Download failed: $_"
        Write-Info "Manual install:"
        Write-Info "  1. https://github.com/asg017/sqlite-vec/releases"
        Write-Info "  2. sqlite-vec-*-loadable-windows-x86_64.zip"
        Write-Info "  3. Extract vec0.dll → $DestPath"
        return $false
    }
}

# ── Banner ────────────────────────────────────────────────────

Write-Host ""
Write-Host "  GeliShell Installer  |  Windows PowerShell" -ForegroundColor Magenta
Write-Host ""

# ── Project root ──────────────────────────────────────────────

$ProjectRoot = $PSScriptRoot
if (-not (Test-Path (Join-Path $ProjectRoot "Cargo.toml"))) {
    Write-Fail "Run this script from the GeliShell project root (where Cargo.toml lives)"
}
Write-Info "project root: $ProjectRoot"

# ── Paths ─────────────────────────────────────────────────────

$UserProfile = $env:USERPROFILE
if (-not $UserProfile) { Write-Fail "USERPROFILE is not set" }

if ($BinDir -eq "") { $BinDir = Join-Path $UserProfile ".local\bin" }
$ConfigRoot = Join-Path $UserProfile ".config\geliShell"
$ModelsDir  = Join-Path $ConfigRoot "models"
$DocsDir    = Join-Path $ConfigRoot "docs"
$Vec0Dest   = Join-Path $ModelsDir  "vec0.dll"
$DocsDbPath = Join-Path $DocsDir    "docs.db"

foreach ($Dir in @($BinDir, $ConfigRoot, $ModelsDir, $DocsDir)) {
    if (-not (Test-Path $Dir)) {
        New-Item -ItemType Directory -Path $Dir -Force | Out-Null
    }
}

# ══════════════════════════════════════════════════════════════
# STEP 1 — geli.exe binary
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Step "installing GeliShell binary..."

$BinarySource = Join-Path $ProjectRoot "target\release\geli_shell.exe"
if (-not (Test-Path $BinarySource)) {
    Write-Host ""
    Write-Warn "Binary not found at: $BinarySource"
    Write-Host "  Run first:  cargo build --release" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

$BinaryDest = Join-Path $BinDir "geli.exe"
Copy-Item -Path $BinarySource -Destination $BinaryDest -Force
Write-Ok "geli.exe → $BinaryDest"

# PATH
$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
$BinDirNorm  = $BinDir.TrimEnd('\')
if (-not ($CurrentPath -split ";" | Where-Object { $_.TrimEnd('\') -eq $BinDirNorm })) {
    [System.Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$BinDirNorm", "User")
    $env:PATH = "$env:PATH;$BinDirNorm"
    Write-Ok "added to user PATH: $BinDirNorm"
    Write-Warn "Restart your terminal for PATH to take effect in new sessions"
} else {
    Write-Info "$BinDir already in PATH"
}

# ══════════════════════════════════════════════════════════════
# STEP 2 — SQLite (sanity check only)
#
# SQLite itself is NOT installed by GeliShell.
# We just check if sqlite3 is on PATH and offer to install it
# if it is missing — it is a system-level dependency.
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Step "checking SQLite..."

$SqliteOk = $null -ne (Get-Command "sqlite3" -ErrorAction SilentlyContinue)

if ($SqliteOk) {
    $SqliteVersion = & sqlite3 --version 2>&1 | Select-Object -First 1
    Write-Ok "sqlite3 found: $SqliteVersion"
} else {
    Write-Warn "sqlite3 not found in PATH"
    Write-Info "SQLite is a runtime dependency for the GeliShell assistant."
    Write-Host ""

    $InstallSqlite = Ask-YesNo "Install SQLite now?"
    if ($InstallSqlite) {
        $SqliteInstalled = $false

        if (-not $SqliteInstalled -and (Get-Command "winget" -ErrorAction SilentlyContinue)) {
            Write-Info "trying: winget install SQLite.SQLite ..."
            try {
                & winget install --id SQLite.SQLite --silent --accept-package-agreements --accept-source-agreements
                if ($null -ne (Get-Command "sqlite3" -ErrorAction SilentlyContinue)) {
                    Write-Ok "SQLite installed via winget"
                    $SqliteInstalled = $true
                    $SqliteOk = $true
                }
            } catch { Write-Warn "winget failed: $_" }
        }

        if (-not $SqliteInstalled -and (Get-Command "choco" -ErrorAction SilentlyContinue)) {
            Write-Info "trying: choco install sqlite ..."
            try {
                & choco install sqlite -y
                if ($null -ne (Get-Command "sqlite3" -ErrorAction SilentlyContinue)) {
                    Write-Ok "SQLite installed via Chocolatey"
                    $SqliteInstalled = $true
                    $SqliteOk = $true
                }
            } catch { Write-Warn "choco failed: $_" }
        }

        if (-not $SqliteInstalled) {
            Write-Warn "Automatic install failed. Download manually:"
            Write-Info "  https://www.sqlite.org/download.html"
            Write-Info "  (sqlite-tools-win-x64-*.zip — add the folder to PATH)"
        }
    } else {
        Write-Info "Skipped. GeliShell core works without SQLite; AI assistant will not work."
    }
}

# ══════════════════════════════════════════════════════════════
# STEP 3 — sqlite-vec (vec0.dll)
#
# sqlite-vec is a SEPARATE project from SQLite.
# Source: https://github.com/asg017/sqlite-vec
#
# Do NOT copy vec0.dll from a SQLite installation folder —
# the correct file comes from sqlite-vec GitHub releases only.
#
# GeliShell loads it at runtime from:
#   %USERPROFILE%\.config\geliShell\models\vec0.dll
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Step "checking sqlite-vec extension (vec0.dll)..."
Write-Info "sqlite-vec is NOT part of SQLite — it is a separate vector-search"
Write-Info "extension: https://github.com/asg017/sqlite-vec"

$Vec0Available = $false

if ((Test-Path $Vec0Dest) -and -not $Force) {
    Write-Ok "vec0.dll already present: $Vec0Dest"
    $Vec0Available = $true
}

# Check local project paths before downloading
if (-not $Vec0Available) {
    $LocalCandidates = @(
        (Join-Path $ProjectRoot "assets\vec0.dll"),
        (Join-Path $ProjectRoot "models\vec0.dll"),
        (Join-Path $ProjectRoot "vec0.dll")
    )
    foreach ($Candidate in $LocalCandidates) {
        if (Test-Path $Candidate) {
            Copy-Item -Path $Candidate -Destination $Vec0Dest -Force
            Write-Ok "vec0.dll found locally → copied from: $Candidate"
            $Vec0Available = $true
            break
        }
    }
}

# Offer GitHub download
if (-not $Vec0Available) {
    Write-Host ""
    Write-Warn "vec0.dll not found locally."
    Write-Host ""
    Write-Host "  GeliShell needs vec0.dll for the AI assistant RAG engine." -ForegroundColor DarkGray
    Write-Host "  It will be placed at: $Vec0Dest" -ForegroundColor DarkGray
    Write-Host ""

    $DownloadVec0 = Ask-YesNo "Download vec0.dll from github.com/asg017/sqlite-vec now?"

    if ($DownloadVec0) {
        $Vec0Available = Invoke-Vec0Download -DestPath $Vec0Dest
    } else {
        Write-Info "Skipped. Install manually:"
        Write-Info "  1. https://github.com/asg017/sqlite-vec/releases"
        Write-Info "  2. Download: sqlite-vec-*-loadable-windows-x86_64.zip"
        Write-Info "  3. Extract vec0.dll and copy it to:"
        Write-Info "     $Vec0Dest"
    }
}

# ══════════════════════════════════════════════════════════════
# STEP 4 — Ollama
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Step "checking Ollama..."

$OllamaOk = $null -ne (Get-Command "ollama" -ErrorAction SilentlyContinue)

if ($OllamaOk) {
    $OllamaVersion = & ollama --version 2>&1 | Select-Object -First 1
    Write-Ok "ollama found: $OllamaVersion"
} else {
    Write-Warn "ollama not found in PATH"
    Write-Info "Ollama is required to generate docs.db (the RAG knowledge base)."
    Write-Info "Install from: https://ollama.com/download"
    Write-Info "Then pull the embedding model:"
    Write-Info "  ollama pull nomic-embed-text"
}

# ══════════════════════════════════════════════════════════════
# STEP 5 — docs.db via build_docs_db
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Step "checking docs.db (RAG knowledge base)..."

$DocsDbOk = (Test-Path $DocsDbPath) -and -not $Force

if ($DocsDbOk) {
    Write-Ok "docs.db already present: $DocsDbPath"
} elseif ($SkipDocs) {
    Write-Info "skipping docs.db generation (--SkipDocs)"
} elseif (-not $Vec0Available) {
    Write-Warn "skipping — vec0.dll not available (required by build_docs_db)"
    Write-Info "Install sqlite-vec first, then run:"
    Write-Info "  cargo run --bin build_docs_db"
} elseif (-not $OllamaOk) {
    Write-Warn "skipping — Ollama not available (required to generate embeddings)"
    Write-Info "Start Ollama and run:"
    Write-Info "  ollama pull nomic-embed-text"
    Write-Info "  cargo run --bin build_docs_db"
} else {
    Write-Host ""
    Write-Host "  docs.db is generated by embedding your markdown docs with Ollama." -ForegroundColor DarkGray
    Write-Host "  Make sure Ollama is running: ollama serve" -ForegroundColor DarkGray
    Write-Host ""
    $RunBuild = Ask-YesNo "Generate docs.db now? (cargo run --bin build_docs_db)"

    if ($RunBuild) {
        Push-Location $ProjectRoot
        try {
            $OldVec0Env = $env:GELI_SQLITE_VEC_PATH
            $env:GELI_SQLITE_VEC_PATH = $Vec0Dest
            Write-Info "GELI_SQLITE_VEC_PATH=$Vec0Dest"
            Write-Host ""
            & cargo run --bin build_docs_db
            $env:GELI_SQLITE_VEC_PATH = $OldVec0Env
            $DocsDbOk = Test-Path $DocsDbPath
            if ($DocsDbOk) {
                Write-Ok "docs.db generated at: $DocsDbPath"
            } else {
                Write-Warn "build_docs_db finished but docs.db not found at expected path"
            }
        } catch {
            Write-Warn "build_docs_db failed: $_"
            Write-Info "Fix the error and re-run:"
            Write-Info "  cd $ProjectRoot"
            Write-Info "  cargo run --bin build_docs_db"
        } finally {
            Pop-Location
        }
    } else {
        Write-Info "Skipped. Run manually when ready:"
        Write-Info "  cargo run --bin build_docs_db"
    }
}

# ══════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════

Write-Host ""
Write-Host "  ──────────────────────────────────────────" -ForegroundColor DarkGray
Write-Host "  GeliShell Installation Summary" -ForegroundColor Magenta
Write-Host "  ──────────────────────────────────────────" -ForegroundColor DarkGray
Write-Host ""

Write-StatusLine -Ok $true          -Label "geli.exe"    -Detail $BinaryDest
Write-StatusLine -Ok $SqliteOk      -Label "SQLite"      -Detail "sqlite3 in PATH"
Write-StatusLine -Ok $Vec0Available -Label "sqlite-vec"  -Detail "vec0.dll — $Vec0Dest"
Write-StatusLine -Ok $OllamaOk      -Label "Ollama"      -Detail "ollama in PATH"
Write-StatusLine -Ok $DocsDbOk      -Label "docs.db"     -Detail $DocsDbPath

Write-Host ""
if ($Vec0Available -and $OllamaOk -and $DocsDbOk) {
    Write-Host "  All components ready." -ForegroundColor Green
} else {
    Write-Host "  GeliShell core is installed and fully functional." -ForegroundColor Green
    Write-Host "  AI assistant features require the missing components above." -ForegroundColor Yellow
}
Write-Host ""
Write-Host "  Open a new terminal and run: geli" -ForegroundColor Cyan
Write-Host ""
