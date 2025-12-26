# build.ps1 - Build script for CI (Collaborative Intelligence CLI)
# Windows PowerShell equivalent of the Makefile

param(
    [Parameter(Position=0)]
    [string]$Target = "build",

    [string]$MSG = "",
    [string]$ARGS = ""
)

$ErrorActionPreference = "Stop"
$BINARY_NAME = "CI"
$INSTALL_DIR = "$env:USERPROFILE\.local\bin"

function Test-RustInstalled {
    try {
        $null = Get-Command cargo -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Install-Rust {
    if (Test-RustInstalled) {
        Write-Host "Rust is already installed:" -ForegroundColor Green
        rustc --version
        cargo --version
    } else {
        Write-Host "Installing Rust via rustup..." -ForegroundColor Yellow

        # Download and run rustup-init.exe
        $rustupUrl = "https://win.rustup.rs/x86_64"
        $tempDir = [System.IO.Path]::GetTempPath()
        $rustupInit = Join-Path $tempDir "rustup-init.exe"

        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupInit
        Start-Process -FilePath $rustupInit -ArgumentList "-y" -Wait -NoNewWindow
        Remove-Item $rustupInit -ErrorAction SilentlyContinue

        # Refresh PATH
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

        Write-Host ""
        Write-Host "Rust installed successfully!" -ForegroundColor Green
        Write-Host "Please restart your terminal or run:" -ForegroundColor Yellow
        Write-Host '  $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"'
    }
}

function Assert-RustInstalled {
    if (-not (Test-RustInstalled)) {
        Write-Host "Error: Rust is not installed." -ForegroundColor Red
        Write-Host "Run: .\build.ps1 install-rust" -ForegroundColor Yellow
        exit 1
    }
}

function Invoke-Build {
    Assert-RustInstalled
    cargo build
}

function Invoke-Release {
    Assert-RustInstalled
    cargo build --release
}

function Invoke-Test {
    Assert-RustInstalled
    cargo test
}

function Invoke-TestVerbose {
    Assert-RustInstalled
    cargo test -- --nocapture
}

function Invoke-Check {
    Assert-RustInstalled
    cargo check
}

function Invoke-Fmt {
    Assert-RustInstalled
    cargo fmt
}

function Invoke-FmtCheck {
    Assert-RustInstalled
    cargo fmt -- --check
}

function Invoke-Lint {
    Assert-RustInstalled
    cargo clippy -- -D warnings
}

function Invoke-LintAll {
    Assert-RustInstalled
    cargo clippy --all-targets --all-features -- -D warnings
}

function Invoke-Clean {
    if (Test-RustInstalled) {
        cargo clean
    } else {
        Remove-Item -Recurse -Force target -ErrorAction SilentlyContinue
    }
}

function Invoke-Install {
    Invoke-Release

    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    }

    Copy-Item "target\release\$BINARY_NAME.exe" $INSTALL_DIR -Force
    if (Test-Path "target\release\cargo-install-ci.exe") {
        Copy-Item "target\release\cargo-install-ci.exe" $INSTALL_DIR -Force
    }

    Write-Host "Installed $BINARY_NAME to $INSTALL_DIR" -ForegroundColor Green

    # Add to PATH if not already there
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$INSTALL_DIR*") {
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$INSTALL_DIR", "User")
        $env:PATH = "$env:PATH;$INSTALL_DIR"
        Write-Host "Added $INSTALL_DIR to PATH" -ForegroundColor Green
        Write-Host "Restart your terminal or run: `$env:PATH = [Environment]::GetEnvironmentVariable('PATH', 'User')" -ForegroundColor Yellow
    }
}

function Invoke-Uninstall {
    Remove-Item "$INSTALL_DIR\$BINARY_NAME.exe" -ErrorAction SilentlyContinue
    Remove-Item "$INSTALL_DIR\cargo-install-ci.exe" -ErrorAction SilentlyContinue
    Write-Host "Uninstalled $BINARY_NAME from $INSTALL_DIR" -ForegroundColor Green
}

function Invoke-Run {
    Assert-RustInstalled
    cargo run
}

function Invoke-RunArgs {
    Assert-RustInstalled
    cargo run -- $ARGS
}

function Invoke-RunRelease {
    Assert-RustInstalled
    cargo run --release
}

function Invoke-Doc {
    Assert-RustInstalled
    cargo doc --no-deps
}

function Invoke-DocOpen {
    Assert-RustInstalled
    cargo doc --no-deps --open
}

function Invoke-Update {
    Assert-RustInstalled
    cargo update
}

function Invoke-Deps {
    Assert-RustInstalled
    cargo tree
}

function Invoke-CI {
    Invoke-FmtCheck
    Invoke-Lint
    Invoke-Test
}

function Invoke-Dev {
    Invoke-Fmt
    Invoke-Check
    Invoke-Build
}

function Invoke-Setup {
    Install-Rust
    if (Test-RustInstalled) {
        cargo build --release
    }
}

function Invoke-Stage {
    git add -A
    Write-Host "Staged all changes" -ForegroundColor Green
    git status --short
}

function Invoke-Commit {
    Invoke-Stage
    if ([string]::IsNullOrEmpty($MSG)) {
        Write-Host "Error: Please provide a commit message with -MSG `"your message`"" -ForegroundColor Red
        exit 1
    }
    git commit -m $MSG
}

function Invoke-Push {
    $branch = git branch --show-current
    git push origin $branch
}

function Invoke-Ship {
    Invoke-Commit
    Invoke-Push
    Write-Host "Changes shipped to GitHub" -ForegroundColor Green
}

function Show-Help {
    Write-Host "CI (Collaborative Intelligence CLI) - Build Commands" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\build.ps1 [target] [-MSG `"message`"] [-ARGS `"args`"]"
    Write-Host ""
    Write-Host "Setup (run first if Rust is not installed):" -ForegroundColor Yellow
    Write-Host "  install-rust  Install Rust via rustup"
    Write-Host "  setup         Install Rust and build the project"
    Write-Host ""
    Write-Host "Build targets:" -ForegroundColor Yellow
    Write-Host "  build         Build in debug mode (default)"
    Write-Host "  release       Build in release mode (optimized)"
    Write-Host "  clean         Remove build artifacts"
    Write-Host ""
    Write-Host "Testing:" -ForegroundColor Yellow
    Write-Host "  test          Run tests"
    Write-Host "  test-verbose  Run tests with output"
    Write-Host "  check         Check code without building"
    Write-Host ""
    Write-Host "Code quality:" -ForegroundColor Yellow
    Write-Host "  fmt           Format code"
    Write-Host "  fmt-check     Check formatting"
    Write-Host "  lint          Run clippy linter"
    Write-Host "  lint-all      Run clippy on all targets"
    Write-Host ""
    Write-Host "Installation:" -ForegroundColor Yellow
    Write-Host "  install       Build release and install to ~/.local/bin"
    Write-Host "  uninstall     Remove from ~/.local/bin"
    Write-Host ""
    Write-Host "Running:" -ForegroundColor Yellow
    Write-Host "  run           Run in debug mode"
    Write-Host "  run-args      Run with args (-ARGS `"...`")"
    Write-Host "  run-release   Run in release mode"
    Write-Host ""
    Write-Host "Documentation:" -ForegroundColor Yellow
    Write-Host "  doc           Build documentation"
    Write-Host "  doc-open      Build and open documentation"
    Write-Host ""
    Write-Host "Git:" -ForegroundColor Yellow
    Write-Host "  stage         Stage all changes"
    Write-Host "  commit        Stage and commit (-MSG `"...`")"
    Write-Host "  push          Push to origin"
    Write-Host "  ship          Stage, commit, push (-MSG `"...`")"
    Write-Host ""
    Write-Host "Other:" -ForegroundColor Yellow
    Write-Host "  update        Update dependencies"
    Write-Host "  deps          Show dependency tree"
    Write-Host "  ci            Full CI check (fmt, lint, test)"
    Write-Host "  dev           Dev workflow (fmt, check, build)"
}

# Main dispatch
switch ($Target.ToLower()) {
    "build"        { Invoke-Build }
    "release"      { Invoke-Release }
    "debug"        { Invoke-Build }
    "test"         { Invoke-Test }
    "test-verbose" { Invoke-TestVerbose }
    "check"        { Invoke-Check }
    "fmt"          { Invoke-Fmt }
    "fmt-check"    { Invoke-FmtCheck }
    "lint"         { Invoke-Lint }
    "lint-all"     { Invoke-LintAll }
    "clean"        { Invoke-Clean }
    "install"      { Invoke-Install }
    "uninstall"    { Invoke-Uninstall }
    "run"          { Invoke-Run }
    "run-args"     { Invoke-RunArgs }
    "run-release"  { Invoke-RunRelease }
    "doc"          { Invoke-Doc }
    "doc-open"     { Invoke-DocOpen }
    "update"       { Invoke-Update }
    "deps"         { Invoke-Deps }
    "ci"           { Invoke-CI }
    "dev"          { Invoke-Dev }
    "setup"        { Invoke-Setup }
    "install-rust" { Install-Rust }
    "stage"        { Invoke-Stage }
    "commit"       { Invoke-Commit }
    "push"         { Invoke-Push }
    "ship"         { Invoke-Ship }
    "help"         { Show-Help }
    default        { Show-Help }
}
