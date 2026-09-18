@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Update-Dawn.ps1" %*
if errorlevel 1 echo Update did not complete. Read the message above.
pause
