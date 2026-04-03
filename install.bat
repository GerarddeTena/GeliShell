@echo off
:: GeliShell Installer — CMD wrapper
:: Launches install.ps1 via PowerShell with process-scoped execution policy.
:: Run from the GeliShell project root.

setlocal EnableDelayedExpansion

echo.
echo   GeliShell Installer
echo   CMD wrapper ^-^> PowerShell
echo.

if not exist "%~dp0Cargo.toml" (
    echo  FAIL: Run this script from the GeliShell project root
    echo        ^(the folder that contains Cargo.toml^)
    exit /b 1
)

set "PS1=%~dp0install.ps1"

if not exist "%PS1%" (
    echo  FAIL: install.ps1 not found at: %PS1%
    exit /b 1
)

:: Prefer PowerShell 7+ (pwsh.exe) over Windows PowerShell 5 (powershell.exe)
:: -ExecutionPolicy Bypass is scoped to this process only — no system change
where pwsh >nul 2>&1
if not errorlevel 1 (
    pwsh.exe ^
        -NoProfile ^
        -ExecutionPolicy Bypass ^
        -File "%PS1%" ^
        %*
    goto :done
)

where powershell >nul 2>&1
if errorlevel 1 (
    echo  FAIL: PowerShell not found. Install PowerShell 5.1 or later.
    exit /b 1
)

powershell.exe ^
    -NoProfile ^
    -ExecutionPolicy Bypass ^
    -File "%PS1%" ^
    %*

:done

set EXIT_CODE=%errorlevel%
if %EXIT_CODE% neq 0 (
    echo.
    echo  Installation failed with exit code %EXIT_CODE%
    exit /b %EXIT_CODE%
)

exit /b 0
