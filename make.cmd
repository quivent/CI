@echo off
:: Bootstrap script for Windows - makes 'make' work even without GNU Make installed
:: Usage: make [target] or just double-click to build

setlocal enabledelayedexpansion

set TARGET=%1
if "%TARGET%"=="" set TARGET=all

:: Check if GNU Make (make.exe) is available
where make.exe >nul 2>&1
if %errorlevel%==0 (
    make.exe %*
    exit /b %errorlevel%
)

:: No GNU Make - handle common targets directly
if "%TARGET%"=="all" goto :setup
if "%TARGET%"=="setup" goto :setup
if "%TARGET%"=="build" goto :build
if "%TARGET%"=="release" goto :release
if "%TARGET%"=="test" goto :test
if "%TARGET%"=="clean" goto :clean
if "%TARGET%"=="install-rust" goto :install_rust
if "%TARGET%"=="help" goto :help

echo Unknown target: %TARGET%
goto :help

:setup
call :check_rust
if %errorlevel%==1 (
    call :install_rust
    echo.
    echo Rust installed. Please restart your terminal and run 'make' again.
    exit /b 0
)
cargo build --release
exit /b %errorlevel%

:build
call :check_rust
cargo build
exit /b %errorlevel%

:release
call :check_rust
cargo build --release
exit /b %errorlevel%

:test
call :check_rust
cargo test
exit /b %errorlevel%

:clean
if exist target rmdir /s /q target
echo Cleaned.
exit /b 0

:install_rust
where cargo >nul 2>&1
if %errorlevel%==0 (
    echo Rust is already installed.
    cargo --version
    exit /b 0
)
echo Installing Rust...
powershell -Command "& {Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '$env:TEMP\rustup-init.exe'; Start-Process -FilePath '$env:TEMP\rustup-init.exe' -ArgumentList '-y' -Wait}"
exit /b 0

:check_rust
where cargo >nul 2>&1
if %errorlevel%==0 exit /b 0
exit /b 1

:help
echo CI Build System
echo.
echo Usage: make [target]
echo.
echo Targets:
echo   (none), all, setup  - Install Rust if needed + build
echo   build               - Debug build
echo   release             - Release build
echo   test                - Run tests
echo   clean               - Remove build artifacts
echo   install-rust        - Install Rust only
echo   help                - Show this help
echo.
echo For full functionality, install GNU Make:
echo   winget install GnuWin32.Make
echo   -or- choco install make
exit /b 0
