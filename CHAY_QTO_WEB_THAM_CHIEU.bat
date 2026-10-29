@echo off
cd /d "%~dp0"
if not exist "QTO_R134_WEB_WORKSPACE\index.html" (echo QTO workspace missing & pause & exit /b 1)
start "" "%~dp0QTO_R134_WEB_WORKSPACE\index.html"
