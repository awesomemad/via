@echo off
:: self-install.bat — Agent self-setup for VIA
:: Detects the platform and installs the right adapter.
setlocal

echo === VIA Self-Install ===
echo.

:: 1. Build everything
echo [1/2] Building binaries...
if not exist "%~dp0target\release\via-mcp.exe" (
    call cargo build --release
    if errorlevel 1 (
        echo ERROR: Build failed.
        exit /b 1
    )
) else (
    echo Already built. Skipping.
)

:: 2. Detect platform and install
echo [2/2] Installing for detected platform...

if exist "%LOCALAPPDATA%\via\plugins\" (
    echo Detected: Hermes Agent (AppData)
    call "%~dp0install-plugin.bat"
    echo.
    echo Done. Tell the user to run: hermes plugins enable via
    exit /b 0
)

if exist "%USERPROFILE%\.hermes\plugins\" (
    echo Detected: Hermes Agent (home dir)
    set PLUGIN_DIR=%USERPROFILE%\.hermes\plugins\via
    if not exist "!PLUGIN_DIR!" mkdir "!PLUGIN_DIR!"
    copy /y "%~dp0via-hermes-plugin\via\__init__.py" "!PLUGIN_DIR!\" >nul
    copy /y "%~dp0via-hermes-plugin\via\plugin.yaml" "!PLUGIN_DIR!\" >nul
    if not exist "!PLUGIN_DIR!\via-mcp.exe" (
        copy /y "%~dp0target\release\via-mcp.exe" "!PLUGIN_DIR!\" >nul
    )
    echo Installed to !PLUGIN_DIR!
    echo Tell the user to run: hermes plugins enable via
    exit /b 0
)

:: opencode, Claude Code, OpenClaw — config files already in repo
if exist "%~dp0.opencode\opencode.json" (
    echo Detected: opencode (config already in repo)
    echo Tell user: restart opencode — tools will appear automatically.
)

if exist "%~dp0.mcp.json" (
    echo Detected: Claude Code / universal MCP (config already in repo)
    echo Tell user: restart Claude Code — tools will appear.
)

if exist "%~dp0openclaw.json" (
    echo Detected: OpenClaw (config already in repo)
    echo Tell user: restart OpenClaw — tools will appear.
)

:: Print generic next-step for any other MCP host
echo.
echo Binaries built. For any MCP host, configure:
echo   command: %~dp0target\release\via-mcp.exe
echo   type: stdio
echo.
echo Done.
