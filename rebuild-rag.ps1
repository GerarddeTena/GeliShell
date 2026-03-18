[CmdletBinding()]
param(
    [string]$DocsDbPath = "$env:USERPROFILE\.config\geliShell\docs\docs.db",
    [string]$VecDllPath = "$env:USERPROFILE\.config\geliShell\models\vec0.dll",
    [int]$BatchSize = 16,
    [string]$Model = "nomic-embed-text",
    [string]$OllamaUrl = "http://127.0.0.1:11434"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$builderExe = Join-Path $repoRoot "target\debug\build_docs_db.exe"
$kbDir = Join-Path $repoRoot "docs\kb"
$scriptingDocs = @(
    (Join-Path $repoRoot "scripting-basico-rag.md"),
    (Join-Path $repoRoot "scripting-medio-rag.md"),
    (Join-Path $repoRoot "scripting-avanzado-rag.md")
)

if (-not (Test-Path -LiteralPath $builderExe)) {
    throw "Missing builder executable: $builderExe. Build it first (cargo build) or ensure target\debug\build_docs_db.exe exists."
}

if (-not (Test-Path -LiteralPath $kbDir)) {
    throw "Missing KB directory: $kbDir"
}

foreach ($doc in $scriptingDocs) {
    if (-not (Test-Path -LiteralPath $doc)) {
        throw "Missing required file: $doc"
    }
}

if (-not (Test-Path -LiteralPath $VecDllPath)) {
    throw "Missing sqlite-vec library: $VecDllPath"
}

$docsDbDir = Split-Path -Parent $DocsDbPath
if ($docsDbDir -and -not (Test-Path -LiteralPath $docsDbDir)) {
    New-Item -ItemType Directory -Path $docsDbDir -Force | Out-Null
}

$stagingDir = Join-Path ([System.IO.Path]::GetTempPath()) ("geli-rag-kb-" + [DateTimeOffset]::UtcNow.ToUnixTimeSeconds())
New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null

try {
    Copy-Item -Path (Join-Path $kbDir "*.md") -Destination $stagingDir -Force
    Copy-Item -Path $scriptingDocs -Destination $stagingDir -Force

    $env:GELI_SQLITE_VEC_PATH = $VecDllPath

    Push-Location $repoRoot
    try {
        & $builderExe `
            --docs-dir $stagingDir `
            --db-path $DocsDbPath `
            --batch-size $BatchSize `
            --model $Model `
            --ollama-url $OllamaUrl

        if ($LASTEXITCODE -ne 0) {
            throw "build_docs_db exited with code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}
finally {
    if (Test-Path -LiteralPath $stagingDir) {
        Remove-Item -Path $stagingDir -Recurse -Force
    }
}

Get-Item -LiteralPath $DocsDbPath | Select-Object FullName, Length, LastWriteTime