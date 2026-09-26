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

echo Starting Quest Pro Touch Plus in development mode...
call npm.cmd run tauri -- dev
if errorlevel 1 goto failed
exit /b 0

:failed
echo.
echo Development launch failed.
pause
exit /b 1
