@echo off
setlocal
pushd "%~dp0"
if not exist "AZSX\App\Python\python.exe" (
  echo [ ERROR ] Extract the complete AZSX archive beside AzerothSurvivors.exe.
  pause
  exit /b 1
)
set "AZSX_LAUNCH_ID=%RANDOM%-%RANDOM%-%RANDOM%"
set "AZSX_RESTARTS=0"
:AZSX_LAUNCH
"AZSX\App\Python\python.exe" -I -B -X utf8 "AZSX\App\Launcher\launch.py" %*
set "AZSX_EXIT=%ERRORLEVEL%"
if not "%AZSX_EXIT%"=="75" goto AZSX_DONE
set /a AZSX_RESTARTS+=1
if %AZSX_RESTARTS% LEQ 4 goto AZSX_LAUNCH
echo [ ERROR ] AZSX could not finish startup. See AZSX/Data/Logs.
set "AZSX_EXIT=1"
:AZSX_DONE
popd
if not "%AZSX_EXIT%"=="0" pause
exit /b %AZSX_EXIT%
