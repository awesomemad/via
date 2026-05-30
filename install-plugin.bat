@echo off
:: install-plugin.bat — Build and install the Hermes VIA plugin
:: Installs to: %LOCALAPPDATA%\via\plugins\via\
setlocal
set PLUGIN_DIR=%LOCALAPPDATA%\via\plugins\via

echo === VIA Hermes Plugin Installer ===
echo.

:: 1. Build via-mcp.exe (skip if already built)
echo [1/3] Checking build...
if not exist "%~dp0target\release\via-mcp.exe" (
    echo Building via-mcp.exe...
    cargo build --package via-mcp --release
    if errorlevel 1 (
        echo ERROR: Build failed. Run setup.bat manually.
        exit /b 1
    )
) else (
    echo via-mcp.exe already built, skipping.
)

:: 2. Create plugin directory
echo [2/3] Creating plugin directory...
if not exist "%PLUGIN_DIR%" mkdir "%PLUGIN_DIR%"

:: 3. Copy plugin files + binary
echo [3/3] Installing to %PLUGIN_DIR%...
copy /y "%~dp0via-hermes-plugin\via\__init__.py" "%PLUGIN_DIR%\" >nul
copy /y "%~dp0via-hermes-plugin\via\plugin.yaml" "%PLUGIN_DIR%\" >nul
copy /y "%~dp0target\release\via-mcp.exe" "%PLUGIN_DIR%\" >nul

if errorlevel 1 (
    echo ERROR: Copy failed.
    exit /b 1
)

echo.
echo === Installed ===
dir /b "%PLUGIN_DIR%"
echo.
echo Next: hermes plugins enable via
echo Or set VIA_MCP_PATH to override binary path.
echo Done.
