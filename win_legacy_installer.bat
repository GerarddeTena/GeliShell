@echo off
:: GeliShell Installer — CMD wrapper
:: Launches install.ps1 via PowerShell with proper execution policy.
:: Run from the GeliShell project root.

setlocal EnableDelayedExpansion

echo.
echo   GeliShell Installer
echo   CMD wrapper ^-^> PowerShell
echo.

:: Verify we are in the project root
if not exist "%~dp0Cargo.toml" (
    echo  FAIL: Run this script from the GeliShell project root
    echo        ^(the folder that contains Cargo.toml^)
    exit /b 1
)

:: Check PowerShell is available
where powershell >nul 2>&1
if errorlevel 1 (
    echo  FAIL: PowerShell not found. Install PowerShell 5.1 or later.
    exit /b 1
)

:: Resolve the .ps1 path relative to this .bat
set "PS1=%~dp0install.ps1"

if not exist "%PS1%" (
    echo  FAIL: install.ps1 not found at: %PS1%
    exit /b 1
)

:: Pass any arguments through to the PowerShell script
:: -ExecutionPolicy Bypass is scoped to this process only — no system change
powershell.exe ^
    -NoProfile ^
    -NonInteractive ^
    -ExecutionPolicy Bypass ^
    -File "%PS1%" ^
    %*

set EXIT_CODE=%errorlevel%

if %EXIT_CODE% neq 0 (
    echo.
    echo  Installation failed with exit code %EXIT_CODE%
    exit /b %EXIT_CODE%
)

exit /b 0