@echo off
setlocal
pushd "%~dp0"
if not exist "AZSX\App\Python\python.exe" (
  echo [ ERROR ] Extract the complete AZSX archive beside AzerothSurvivors.exe.
  pause
  exit /b 1
)
:AZSX_LAUNCH
"AZSX\App\Python\python.exe" -I -B -X utf8 "AZSX\App\Launcher\launch.py" %*
set "AZSX_EXIT=%ERRORLEVEL%"
if "%AZSX_EXIT%"=="75" goto AZSX_LAUNCH
popd
if not "%AZSX_EXIT%"=="0" pause
exit /b %AZSX_EXIT%
