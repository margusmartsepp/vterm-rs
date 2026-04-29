@echo off
echo Stopping VTerm instances...
taskkill /F /IM vterm.exe /T 2>nul
taskkill /F /IM vterm-ctrl.exe /T 2>nul
taskkill /F /IM vterm-mcp.exe /T 2>nul
echo Done.
