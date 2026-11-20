@echo off
rem Polyglotic brush engine: the two-minute test for a PC with integrated graphics.
rem It times the ink simulation on every graphics adapter, then shows a washed
rem rectangle with a moving line for about 20 seconds. It writes numbers and
rem adapter names to brush-floor-report.txt, here. Nothing is sent anywhere.
cd /d "%~dp0"
brush.exe floor --out brush-floor-report.txt
echo.
echo Done. Please send back brush-floor-report.txt from this folder.
pause
