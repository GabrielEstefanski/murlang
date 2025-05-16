@echo off
chcp 65001 > nul
type "%~dp0banner.txt"           
echo.

if not exist "%~dp0..\bin" mkdir "%~dp0..\bin"

echo Mrglgrgl... Compiling executable...
pushd "%~dp0.."
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Aaaaaughibbrgubugbugrguburglegrrr! Error compiling!
    exit /b 1
)
popd

echo Copying executable...
copy /Y "%~dp0..\target\release\mur_lang.exe" "%~dp0..\bin\murlang.exe" > nul

echo Creating wrapper...
(
echo @echo off
echo set MURLANG_HOME=%%~dp0..
echo.
echo if "%%1"=="run" (
echo     if "%%2"=="" (
echo         echo Mrglgrgl! Specify a .mur file to execute!
echo         exit /b 1
echo     ^)
echo     "%%MURLANG_HOME%%\bin\murlang.exe" "%%2"
echo     exit /b %%ERRORLEVEL%%
echo ^)
echo.
echo if "%%1"=="help" (
echo     "%%MURLANG_HOME%%\bin\murlang.exe" help
echo     exit /b 0
echo ^)
echo.
echo if "%%1"=="--version" (
echo     "%%MURLANG_HOME%%\bin\murlang.exe" --version
echo     exit /b 0
echo ^)
echo.
echo if "%%1"=="-V" (
echo     "%%MURLANG_HOME%%\bin\murlang.exe" --version
echo     exit /b 0
echo ^)
echo.
echo echo Mrglglgl? Unknown command. Use 'mrgl help' for help.
echo exit /b 1
) > "%~dp0..\bin\mrgl.bat"

echo Adding to PATH...
set "BIN_PATH=%~dp0..\bin"
setx MURLANG_HOME "%~dp0.." /M
setx PATH "%BIN_PATH%;%PATH%" /M

set "PATH=%BIN_PATH%;%PATH%"

reg add "HKCR\.mur" /ve /d "MurlangFile" /f
reg add "HKCR\MurlangFile" /ve /d "Murlang File" /f
reg add "HKCR\MurlangFile\DefaultIcon" /ve /d "%~dp0..\murlang.ico" /f
reg add "HKCR\MurlangFile\shell\open\command" /ve /d "\"%~dp0..\bin\murlang.exe\" \"%%1\"" /f

echo.
echo Mglrmglmglmgl! Installation completed!
echo.
echo To use Murlang, try:
echo     mrgl help
echo.
echo If the command is not recognized, try running:
echo     %~dp0..\bin\mrgl.bat help
echo.
echo Aaaaaughibbrgubugbugrguburgle!