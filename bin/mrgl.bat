@echo off
set MURLANG_HOME=%~dp0..

if "%1"=="run" (
    if "%2"=="" (
        echo Mrglgrgl! Specify a .mur file to execute!
        exit /b 1
    )
    "%MURLANG_HOME%\bin\murlang.exe" "%2"
    exit /b %ERRORLEVEL%
)

if "%1"=="help" (
    echo Mrglglglgl! Commands available:
    echo   mrgl run <file.mur>    - Runs a Murlang program
    echo   mrgl help             - Shows this help
    exit /b 0
)

echo Mrglglgl? Unknown command. Use 'mrgl help' for help.
exit /b 1
