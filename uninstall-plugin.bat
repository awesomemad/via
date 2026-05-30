@echo off
:: uninstall-plugin.bat — Remove the Hermes VIA plugin
set PLUGIN_DIR=%LOCALAPPDATA%\via\plugins\via

echo === Uninstalling VIA Hermes Plugin ===
echo.

if exist "%PLUGIN_DIR%" (
    echo Removing %PLUGIN_DIR%
    rmdir /s /q "%PLUGIN_DIR%"
    echo Done.
) else (
    echo Plugin not found at %PLUGIN_DIR%
)

echo.
echo If you also want to remove config, delete:
echo   %%LOCALAPPDATA%%\via\  (keeps any other VIA files)
echo.
