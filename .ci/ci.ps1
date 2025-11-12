<#
Run the project's CI checks locally (PowerShell).

Usage:
  # run in repository root
  .\.ci\run-ci.ps1

This mirrors the GitHub Actions `ci.yml` steps:
  - cargo fmt --check
  - cargo check (all targets & features)
  - cargo clippy (deny warnings)
  - cargo build --release
  - cargo test
  - cargo doc --no-deps
  - cargo-audit (install if missing)

Notes:
  - This script runs on Windows PowerShell. On Unix/macOS use `.ci/run-ci.sh`.
  - Some commands (clippy with -D warnings) will fail the run if any warnings remain.
  - Running with `--all-features` may take longer but reduces false-positives.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Run-Step {
  param(
    [string]$Name,
    [string]$Cmd
  )

  Write-Host "=== $Name ===" -ForegroundColor Yellow
  Invoke-Expression $Cmd
  $code = $LASTEXITCODE
  if ($code -ne 0) {
    Write-Host "Step '$Name' failed with exit code $code" -ForegroundColor Red
    exit $code
  }
}

Write-Host "Starting local CI checks..." -ForegroundColor Cyan

Run-Step "Format check" 'cargo fmt --all -- --check'

# Run upgrade & build early to fail fast on dependency or build regressions
Write-Host "=== Upgrade & build ===" -ForegroundColor Yellow
& .\.ci\build.ps1
$rc = $LASTEXITCODE
if ($rc -ne 0) {
  Write-Host "Upgrade & build failed (exit code $rc). Aborting CI. See .ci\\build.ps1 output for details." -ForegroundColor Red
  exit $rc
}

Run-Step "Cargo check (all targets & features)" 'cargo check --all-targets --all-features --verbose'
Run-Step "Clippy (deny warnings)" 'cargo clippy --all-targets --all-features -- -D warnings'
Run-Step "Release build" 'cargo build --release --all-targets --all-features --verbose'
Run-Step "Tests" 'cargo test --all-features --all-targets --verbose'
Run-Step "Docs (deny doc warnings)" 'cargo doc --no-deps --all-features --verbose'

Run-Step "Install nightly toolchain" 'rustup toolchain install nightly'

# Ensure cargo-udeps is installed (needed for workspace-wide unused-deps analysis)
if (-not (Get-Command cargo-udeps -ErrorAction SilentlyContinue)) {
  Write-Host "cargo-udeps not found; installing..." -ForegroundColor Cyan
  Invoke-Expression 'cargo install --locked cargo-udeps'
  if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to install cargo-udeps" -ForegroundColor Red
    exit $LASTEXITCODE
  }
}

Write-Host "=== Run cargo +nightly udeps (check for unused deps) ===" -ForegroundColor Yellow
$stdoutFile = [System.IO.Path]::GetTempFileName()
$stderrFile = [System.IO.Path]::GetTempFileName()
$args = @('+nightly','udeps','--all-targets','--all-features','--workspace')
$proc = Start-Process -FilePath 'cargo' -ArgumentList $args -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile

$stdOut = Get-Content -Raw -Path $stdoutFile -ErrorAction SilentlyContinue
$stdErr = Get-Content -Raw -Path $stderrFile -ErrorAction SilentlyContinue
Remove-Item $stdoutFile, $stderrFile -ErrorAction SilentlyContinue

$udepsOutput = $stdOut + "`n" + $stdErr
Write-Host $udepsOutput

if ($proc.ExitCode -ne 0) {
    Write-Host "cargo-udeps failed with exit code $($proc.ExitCode)" -ForegroundColor Red
    exit $proc.ExitCode
}

if ($udepsOutput -notmatch 'All deps seem to have been used\.') {
  Write-Host "cargo-udeps reported unused dependencies; failing local CI." -ForegroundColor Red
  exit 1
}

Write-Host "=== Install & run cargo-audit ===" -ForegroundColor Yellow
try {
  cargo audit --version | Out-Null
  Write-Host "cargo-audit is installed" -ForegroundColor Cyan
} catch {
  Write-Host "cargo-audit not found; installing..." -ForegroundColor Cyan
  Invoke-Expression 'cargo install --locked cargo-audit'
  if ($LASTEXITCODE -ne 0) {
    Write-Host "Failed to install cargo-audit" -ForegroundColor Red
    exit $LASTEXITCODE
  }
}

Run-Step "cargo audit" 'cargo audit'

Write-Host "All CI checks completed." -ForegroundColor Green
