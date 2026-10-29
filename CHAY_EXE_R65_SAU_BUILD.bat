@echo off
setlocal
cd /d "%~dp0"
if not exist "cadcraft.exe" (echo Missing R65 built EXE. & pause & exit /b 1)
start "" "%~dp0cadcraft.exe"
