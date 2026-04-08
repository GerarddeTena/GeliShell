#Requires -Version 5.1
<#
.SYNOPSIS
    GeliShell installer for Windows (PowerShell)

.DESCRIPTION
    Installs GeliShell and all required runtime dependencies:

      1. geli.exe + gerisabet.exe  ->  %USERPROFILE%\.local\bin\
      2. docs.db (pre-generated)   ->  %USERPROFILE%\.config\geliShell\docs\docs.db
      3. sqlite-vec (vec0.dll)     ->  %USERPROFILE%\.config\geliShell\models\vec0.dll

    IMPORTANT: docs.db is a pre-generated release artifact.
    It must be present at assets\docs.db in the project root before running
    this installer.  It is NOT built at install time.
    Developer build step: cargo run --bin build_docs_db

    IMPORTANT: sqlite-vec is NOT part of SQLite.
    It is a separate vector-search extension: https://github.com/asg017/sqlite-vec

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
# sqlite-vec downloader
# Returns $true on success, $false on any recoverable failure.
# ══════════════════════════════════════════════════════════════
function Invoke-Vec0Download
{
    param(
        [string]$DestPath,
        [string]$Arch
    )

    # sqlite-vec has no upstream Windows ARM64 build.
    if ($Arch -eq 'aarch64')
    {
        Write-Warn "sqlite-vec has no official Windows ARM64 build."
        Write-Info "See: https://github.com/asg017/sqlite-vec/releases"
        Write-Info "The AI assistant RAG engine will not work on this architecture."
        return $false
    }

    Write-Info "fetching latest release info from GitHub API..."

    $ApiHeaders = @{
        'User-Agent' = 'GeliShell-Installer/1.0'
        'Accept'     = 'application/vnd.github+json'
    }

    $Release = Invoke-SafeRestMethod `
        -Uri     'https://api.github.com/repos/asg017/sqlite-vec/releases/latest' `
        -Headers $ApiHeaders `
        -TimeoutSec 15

    $Tag = $Release.tag_name
    Write-Info "latest sqlite-vec release: $Tag"

    # Windows loadable is .tar.gz (not .zip — the .zip does not exist upstream)
    $Asset = $Release.assets |
        Where-Object { $_.name -match 'loadable-windows-x86_64\.tar\.gz$' } |
        Select-Object -First 1

    if (-not $Asset)
    {
        Write-Warn "Windows x64 loadable .tar.gz not found in release $Tag"
        Write-Info "Available assets:"
        $Release.assets | ForEach-Object { Write-Info "  $($_.name)" }
        return $false
    }

    $AssetName   = $Asset.name
    $TempArchive = Join-Path $env:TEMP "sqlite-vec-$Tag-$AssetName"
    $TempDir     = Join-Path $env:TEMP "sqlite-vec-extract-$Tag"

    # ── Fetch checksums.txt for SHA-256 verification ──────────
    # sqlite-vec publishes checksums.txt alongside every release.
    $ChecksumsUrl  = "https://github.com/asg017/sqlite-vec/releases/download/$Tag/checksums.txt"
    $TempChecksums = Join-Path $env:TEMP "sqlite-vec-$Tag-checksums.txt"
    $ExpectedHash  = $null

    try
    {
        Invoke-SafeWebRequest -Uri $ChecksumsUrl -OutFile $TempChecksums -TimeoutSec 15
        $Lines     = Get-Content $TempChecksums -ErrorAction Stop
        $MatchLine = $Lines | Where-Object { $_ -match [regex]::Escape($AssetName) }
        if ($MatchLine)
        {
            $ExpectedHash = ($MatchLine.Trim() -split '\s+')[1]
            Write-Info "found SHA-256 for $AssetName in checksums.txt"
        }
        else
        {
            Write-Warn "asset not found in checksums.txt — SHA-256 verification will be skipped"
        }
    }
    catch
    {
        Write-Warn "could not fetch checksums.txt — SHA-256 verification will be skipped"
    }
    finally
    {
        Remove-Item $TempChecksums -Force -ErrorAction SilentlyContinue
    }

    # ── Download archive ──────────────────────────────────────
    Write-Info "downloading: $AssetName ..."
    Invoke-SafeWebRequest -Uri $Asset.browser_download_url -OutFile $TempArchive -TimeoutSec 120

    # ── Verify SHA-256 (skipped when ExpectedHash is null/empty) ─
    Confirm-Checksum -FilePath $TempArchive -Expected "$ExpectedHash"

    # ── Extract .tar.gz via tar.exe (Windows 10 1803+ / Server 2019+) ─
    if (Test-Path $TempDir) { Remove-Item $TempDir -Recurse -Force }
    New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

    if (-not (Get-Command 'tar' -ErrorAction SilentlyContinue))
    {
        Write-Warn "tar.exe not found — Windows 10 1803+ or Git for Windows is required"
        Write-Info "Alternative: install Git for Windows (includes tar)"
        Remove-Item $TempArchive -Force -ErrorAction SilentlyContinue
        Remove-Item $TempDir    -Recurse -Force -ErrorAction SilentlyContinue
        return $false
    }

    & tar -xzf $TempArchive -C $TempDir
    if ($LASTEXITCODE -ne 0)
    {
        Write-Warn "tar extraction failed (exit $LASTEXITCODE)"
        Remove-Item $TempArchive -Force -ErrorAction SilentlyContinue
        Remove-Item $TempDir    -Recurse -Force -ErrorAction SilentlyContinue
        return $false
    }

    $Dll = Get-ChildItem -Path $TempDir -Recurse -Filter 'vec0.dll' |
           Select-Object -First 1
    if (-not $Dll)
    {
        $Dll = Get-ChildItem -Path $TempDir -Recurse -Filter '*.dll' |
               Select-Object -First 1
    }

    if (-not $Dll)
    {
        Write-Warn "vec0.dll not found inside the downloaded archive."
        Remove-Item $TempArchive -Force   -ErrorAction SilentlyContinue
        Remove-Item $TempDir     -Recurse -Force -ErrorAction SilentlyContinue
        return $false
    }

    $Parent = Split-Path $DestPath -Parent
    if (-not (Test-Path $Parent))
    {
        New-Item -ItemType Directory -Path $Parent -Force | Out-Null
    }
    Copy-Item -Path $Dll.FullName -Destination $DestPath -Force
    Register-Rollback -Path $DestPath

    Remove-Item $TempArchive -Force   -ErrorAction SilentlyContinue
    Remove-Item $TempDir     -Recurse -Force -ErrorAction SilentlyContinue

    Write-Ok "vec0.dll installed at: $DestPath"
    return $true
}

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

    # ── Detect architecture ───────────────────────────────────
    $Arch = Get-InstallerArchitecture
    Write-Info "architecture: $Arch"

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
    $Vec0Dest    = Join-Path $ModelsDir    'vec0.dll'
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
    # STEP 3 — SQLite (sanity check only)
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Step "checking SQLite..."

    $SqliteOk = $null -ne (Get-Command 'sqlite3' -ErrorAction SilentlyContinue)

    if ($SqliteOk)
    {
        $SqliteVersion = & sqlite3 --version 2>&1 | Select-Object -First 1
        Write-Ok "sqlite3 found: $SqliteVersion"
    }
    else
    {
        Write-Warn "sqlite3 not found in PATH"
        Write-Info "SQLite is a runtime dependency for the GeliShell AI assistant."
        Write-Host ""

        if (Ask-YesNo -Question "Install SQLite now?")
        {
            $SqliteInstalled = $false

            if (-not $SqliteInstalled -and (Get-Command 'winget' -ErrorAction SilentlyContinue))
            {
                Write-Info "trying: winget install SQLite.SQLite ..."
                try
                {
                    & winget install --id SQLite.SQLite --silent `
                        --accept-package-agreements --accept-source-agreements
                    if (Get-Command 'sqlite3' -ErrorAction SilentlyContinue)
                    {
                        Write-Ok "SQLite installed via winget"
                        $SqliteInstalled = $true
                        $SqliteOk        = $true
                    }
                }
                catch { Write-Warn "winget failed: $_" }
            }

            if (-not $SqliteInstalled -and (Get-Command 'choco' -ErrorAction SilentlyContinue))
            {
                Write-Info "trying: choco install sqlite ..."
                try
                {
                    & choco install sqlite -y
                    if (Get-Command 'sqlite3' -ErrorAction SilentlyContinue)
                    {
                        Write-Ok "SQLite installed via Chocolatey"
                        $SqliteInstalled = $true
                        $SqliteOk        = $true
                    }
                }
                catch { Write-Warn "choco failed: $_" }
            }

            if (-not $SqliteInstalled)
            {
                Write-Warn "Automatic install failed. Download manually:"
                Write-Info "  https://www.sqlite.org/download.html"
                Write-Info "  (sqlite-tools-win-x64-*.zip — add the folder to PATH)"
            }
        }
        else
        {
            Write-Info "Skipped. GeliShell core works without SQLite; AI assistant will not."
        }
    }

    # ══════════════════════════════════════════════════════════
    # STEP 4 — sqlite-vec (vec0.dll)
    #
    # sqlite-vec is a SEPARATE project from SQLite.
    # Source: https://github.com/asg017/sqlite-vec
    #
    # GeliShell loads it at runtime from:
    #   %USERPROFILE%\.config\geliShell\models\vec0.dll
    # ══════════════════════════════════════════════════════════
    Write-Host ""
    Write-Step "checking sqlite-vec extension (vec0.dll)..."
    Write-Info "sqlite-vec is NOT part of SQLite — separate vector-search extension"
    Write-Info "source: https://github.com/asg017/sqlite-vec"

    $Vec0Available = $false

    if ((Test-Path $Vec0Dest) -and -not $Force)
    {
        Write-Ok "vec0.dll already present: $Vec0Dest"
        $Vec0Available = $true
    }

    if (-not $Vec0Available)
    {
        $Vec0Candidates = @(
            (Join-Path $ProjectRoot 'assets\vec0.dll'),
            (Join-Path $ProjectRoot 'models\vec0.dll'),
            (Join-Path $ProjectRoot 'vec0.dll')
        )
        foreach ($Candidate in $Vec0Candidates)
        {
            if (Test-Path $Candidate)
            {
                Copy-Item -Path $Candidate -Destination $Vec0Dest -Force
                Register-Rollback -Path $Vec0Dest
                Write-Ok "vec0.dll found locally -> copied from: $Candidate"
                $Vec0Available = $true
                break
            }
        }
    }

    if (-not $Vec0Available)
    {
        Write-Host ""
        Write-Warn "vec0.dll not found locally."
        Write-Host ""
        Write-Info "GeliShell needs vec0.dll for the AI assistant RAG engine."
        Write-Info "It will be placed at: $Vec0Dest"
        Write-Host ""

        if (Ask-YesNo -Question "Download vec0.dll from github.com/asg017/sqlite-vec now?")
        {
            $Vec0Available = Invoke-Vec0Download -DestPath $Vec0Dest -Arch $Arch
        }
        else
        {
            Write-Info "Skipped. Install manually:"
            Write-Info "  1. https://github.com/asg017/sqlite-vec/releases"
            Write-Info "  2. Download: sqlite-vec-*-loadable-windows-x86_64.tar.gz"
            Write-Info "  3. Extract vec0.dll and copy to:"
            Write-Info "     $Vec0Dest"
        }
    }

    # ══════════════════════════════════════════════════════════
    # STEP 5 — Post-installation verification
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

    Write-StatusLine -Ok $true           -Label 'geli.exe'      -Detail $GeliBin
    Write-StatusLine -Ok $true           -Label 'gerisabet.exe' -Detail (Join-Path $BinDir 'gerisabet.exe')
    Write-StatusLine -Ok $SqliteOk       -Label 'SQLite'        -Detail 'sqlite3 in PATH'
    Write-StatusLine -Ok $Vec0Available  -Label 'sqlite-vec'    -Detail "vec0.dll — $Vec0Dest"
    Write-StatusLine -Ok $DocsDbOk       -Label 'docs.db'       -Detail $DocsDbDest

    Write-Host ""
    if ($Vec0Available -and $DocsDbOk)
    {
        Write-Host "  All components ready." -ForegroundColor Green
    }
    else
    {
        Write-Host "  GeliShell core is installed and fully functional." -ForegroundColor Green
        Write-Host "  AI assistant features require the missing components above." -ForegroundColor Yellow
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
