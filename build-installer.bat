@echo off
setlocal
cd /d "%~dp0" || exit /b 1

where npm.cmd >nul 2>&1
if errorlevel 1 (
  echo Node.js and npm are required. Install them and try again.
  pause
  exit /b 1
)

if not exist "node_modules\" (
  echo Installing dependencies...
  call npm.cmd ci
  if errorlevel 1 goto failed
)

echo Building the Windows installer...
call npm.cmd run tauri -- build --bundles nsis
if errorlevel 1 goto failed

echo.
echo Installer created in src-tauri\target\release\bundle\nsis\
pause
exit /b 0

:failed
echo.
echo Build failed.
pause
exit /b 1
