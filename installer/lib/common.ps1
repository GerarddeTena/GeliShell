# installer/lib/common.ps1 — Shared functions for GeliShell Windows install script.
#
# Dot-source this file from install.ps1:
#   . (Join-Path $PSScriptRoot 'installer\lib\common.ps1')
#
# Requires: PowerShell 5.1+
#
# Provides:
#   Logging      : Write-Step, Write-Ok, Write-Warn, Write-Info, Write-Fail
#   Prompts      : Ask-YesNo
#   Rollback     : Register-Rollback, Invoke-Rollback   ($script:RollbackFiles)
#   Checksums    : Confirm-Checksum
#   Web requests : Invoke-SafeWebRequest, Invoke-SafeRestMethod
#   Platform     : Get-InstallerArchitecture
#   Summary      : Write-StatusLine

Set-StrictMode -Version Latest

# ── Logging ───────────────────────────────────────────────────
function Write-Step { param([string]$Msg) Write-Host "  --> $Msg" -ForegroundColor Cyan }
function Write-Ok   { param([string]$Msg) Write-Host "  [OK] $Msg" -ForegroundColor Green }
function Write-Warn { param([string]$Msg) Write-Host " [WARN] $Msg" -ForegroundColor Yellow }
function Write-Info { param([string]$Msg) Write-Host "       $Msg" -ForegroundColor DarkGray }

# Write-Fail throws so the main script's try/catch can run Invoke-Rollback
# before exiting.  Do NOT call exit 1 here — that would bypass the catch block.
function Write-Fail
{
    param([string]$Msg)
    Write-Host " [ERROR] $Msg" -ForegroundColor Red
    throw $Msg
}

# ── Interactive prompt ────────────────────────────────────────
function Ask-YesNo
{
    param(
        [string]$Question,
        [bool]$Default = $true
    )
    $hint = if ($Default) { '[Y/n]' } else { '[y/N]' }
    Write-Host "  $Question $hint " -ForegroundColor Cyan -NoNewline
    $answer = (Read-Host).Trim().ToLower()
    if ($answer -eq '') { return $Default }
    return ($answer -eq 'y' -or $answer -eq 'yes')
}

# ── Rollback tracker ─────────────────────────────────────────
# Files registered here are deleted by Invoke-Rollback().
# Register every file copied during installation with Register-Rollback.
$script:RollbackFiles = [System.Collections.Generic.List[string]]::new()
$script:RollbackDone  = $false

# Register-Rollback -Path <file_path>
function Register-Rollback
{
    param([string]$Path)
    $script:RollbackFiles.Add($Path)
}

# Invoke-Rollback
# Idempotent — safe to call multiple times (only acts once).
function Invoke-Rollback
{
    if ($script:RollbackDone) { return }
    $script:RollbackDone = $true
    if ($script:RollbackFiles.Count -eq 0) { return }
    Write-Warn "Rolling back installation..."
    foreach ($f in $script:RollbackFiles)
    {
        if (Test-Path $f)
        {
            Remove-Item -Path $f -Force -ErrorAction SilentlyContinue
            Write-Info "removed: $f"
        }
    }
}

# ── SHA-256 verification ──────────────────────────────────────
# Confirm-Checksum -FilePath <path> -Expected <hash>
#
# If Expected is empty or whitespace the function prints a warning and
# returns (skip mode).  This happens when checksums.txt could not be fetched.
#
# TODO: populate Expected from the release pipeline so the empty-string
#       path is never taken in production.
function Confirm-Checksum
{
    param(
        [string]$FilePath,
        [AllowEmptyString()][string]$Expected
    )

    if ([string]::IsNullOrWhiteSpace($Expected))
    {
        Write-Warn "SHA-256 checksum not available — skipping verification"
        # TODO: populate from release pipeline
        return
    }

    $actual = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
    $exp    = $Expected.ToLower().Trim()

    if ($actual -eq $exp)
    {
        Write-Ok "SHA-256 verified: $(Split-Path $FilePath -Leaf)"
    }
    else
    {
        # Propagates to the main script's catch block → Invoke-Rollback + exit 1
        throw "SHA-256 mismatch for $(Split-Path $FilePath -Leaf)`n  expected: $exp`n  actual:   $actual"
    }
}

# ── Web request helpers ───────────────────────────────────────
# Invoke-SafeWebRequest
# Always passes -UseBasicParsing so the call works on PS5 without the
# Internet Explorer engine (common on Windows Server / headless machines).
function Invoke-SafeWebRequest
{
    param(
        [string]$Uri,
        [string]$OutFile,
        [int]$TimeoutSec    = 120,
        [hashtable]$Headers = @{}
    )
    $params = @{
        Uri             = $Uri
        OutFile         = $OutFile
        TimeoutSec      = $TimeoutSec
        UseBasicParsing = $true
    }
    if ($Headers.Count -gt 0) { $params['Headers'] = $Headers }
    Invoke-WebRequest @params
}

# Invoke-SafeRestMethod
# Thin wrapper for JSON API calls.
# Note: Invoke-RestMethod does NOT accept -UseBasicParsing (that flag is
# specific to Invoke-WebRequest).
function Invoke-SafeRestMethod
{
    param(
        [string]$Uri,
        [int]$TimeoutSec    = 15,
        [hashtable]$Headers = @{}
    )
    $params = @{
        Uri        = $Uri
        TimeoutSec = $TimeoutSec
    }
    if ($Headers.Count -gt 0) { $params['Headers'] = $Headers }
    Invoke-RestMethod @params
}

# ── Architecture detection ────────────────────────────────────
# Get-InstallerArchitecture
# Normalises $env:PROCESSOR_ARCHITECTURE to a canonical token.
# Returns: x86_64 | aarch64 | x86 | unknown
function Get-InstallerArchitecture
{
    switch ($env:PROCESSOR_ARCHITECTURE)
    {
        'AMD64' { return 'x86_64'  }
        'ARM64' { return 'aarch64' }
        'x86'   { return 'x86'     }
        default { return 'unknown' }
    }
}

# ── Summary line ──────────────────────────────────────────────
function Write-StatusLine
{
    param(
        [bool]$Ok,
        [string]$Label,
        [string]$Detail = ''
    )
    $icon  = if ($Ok) { '  [OK]' } else { '  [--]' }
    $color = if ($Ok) { 'Green' } else { 'DarkGray' }
    Write-Host "$icon $Label" -ForegroundColor $color
    if ($Detail) { Write-Host "       $Detail" -ForegroundColor DarkGray }
}
