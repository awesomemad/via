@echo off
echo === VIA - Sovereign GUI Engine Build ===
echo.

echo [1/4] Building engine core (release)...
cargo build --package via-engine --release
if %ERRORLEVEL% neq 0 (
    echo ERROR: Engine build failed!
    exit /b 1
)

echo [2/4] Building CLI adapter (release)...
cargo build --package via-cli --release
if %ERRORLEVEL% neq 0 (
    echo ERROR: CLI build failed!
    exit /b 1
)

echo [3/4] Building MCP adapter (release)...
cargo build --package via-mcp --release
if %ERRORLEVEL% neq 0 (
    echo ERROR: MCP build failed!
    exit /b 1
)

echo [4/4] Building C-ABI native plugin (release)...
cargo build --package via-cdylib --release
if %ERRORLEVEL% neq 0 (
    echo ERROR: cdylib build failed!
    exit /b 1
)

echo === Build complete! ===
echo.
echo === Output Files ===
echo Engine lib:  target\release\via_engine.lib
echo CLI:         target\release\via.exe
echo MCP:         target\release\via-mcp.exe
echo C-ABI:       target\release\via_plugin.dll
echo.
echo === Adapter Coverage ===
echo.
echo  Adapter      Platforms
echo  -----------  -------------------------------------------
echo  via.exe      Any shell, any agent (bash/run fallback)
echo  via-mcp.exe  opencode, Cursor, Claude Desktop,
echo               VS Code Copilot, Zed, Windsurf, Cline,
echo               Continue.dev, any MCP host
echo  via_plugin   C/Python/Rust/.NET hosts via LoadLibrary
echo               (future: Hermes, A2A, opencode plugin)
echo.
echo === MCP Config (opencode.json / cursor / claude_desktop_config) ===
echo.
echo {
echo   "mcp": {
echo     "via": {
echo       "command": "%CD%\target\release\via-mcp.exe",
echo       "type": "stdio",
echo       "description": "Sovereign GUI Engine - total Windows GUI control"
echo     }
echo   }
echo }
echo.
echo === Quick Test ===
echo Run: target\release\via.exe dump
echo.
echo Done.
