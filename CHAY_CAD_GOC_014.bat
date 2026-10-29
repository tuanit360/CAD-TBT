@echo off
cd /d "%~dp0"
if not exist "CAD_ORIGINAL_014_REFERENCE\cadcraft.exe" (echo EXE goc not found & pause & exit /b 1)
start "" "%~dp0CAD_ORIGINAL_014_REFERENCE\cadcraft.exe"
