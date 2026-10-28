@echo off
setlocal
pushd "%~dp0"
if not exist "ASX\Runtime\python.exe" (
  echo [ ERROR ] Extract the complete ASX archive beside AzerothSurvivors.exe.
  pause
  exit /b 1
)
:ASX_LAUNCH
"ASX\Runtime\python.exe" -I -B -X utf8 "ASX\Client\launch.py" %*
set "ASX_EXIT=%ERRORLEVEL%"
if "%ASX_EXIT%"=="75" goto ASX_LAUNCH
popd
if not "%ASX_EXIT%"=="0" pause
exit /b %ASX_EXIT%
