@echo off
echo Registering Pirate Downloader Native Host...

:: Set variables
set HOST_NAME=com.piratedownloader.host
set MANIFEST_PATH=%~dp0host_manifest.json
set EXE_PATH=%~dp0src-tauri\target\debug\pirate-host.exe

:: Build the host first to ensure it exists
echo Building pirate-host...
cd src-tauri
cargo build -p pirate-host
cd ..

:: Check if build succeeded
if not exist "%EXE_PATH%" (
    echo Build failed! pirate-host.exe not found at %EXE_PATH%
    pause
    exit /b 1
)

:: Update manifest with absolute path to exe
echo Updating manifest path...
powershell -Command "(Get-Content host_manifest.json) -replace '\"path\": \".*\"', '\"path\": \"%EXE_PATH:\=\\%\"' | Set-Content host_manifest.json"

:: Registry Key (Current User)
set REG_KEY=HKCU\Software\Google\Chrome\NativeMessagingHosts\%HOST_NAME%

:: Add to Registry
echo Adding to Registry: %REG_KEY%
reg add "%REG_KEY%" /ve /t REG_SZ /d "%MANIFEST_PATH%" /f

echo.
echo ========================================================
echo Registration Complete!
echo.
echo NOTE: You must update the 'allowed_origins' in host_manifest.json
echo with your actual Chrome Extension ID after loading it.
echo ========================================================
echo.
pause
