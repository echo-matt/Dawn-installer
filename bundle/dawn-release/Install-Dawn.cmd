@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Install-Dawn.ps1" %*
if errorlevel 1 echo Installation did not complete. Read the message above.
pause
