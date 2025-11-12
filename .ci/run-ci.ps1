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
Run-Step "Cargo check (all targets & features)" 'cargo check --all-targets --all-features --verbose'
Run-Step "Clippy (deny warnings)" 'cargo clippy --all-targets --all-features -- -D warnings'
Run-Step "Release build" 'cargo build --release --all-targets --all-features --verbose'
Run-Step "Tests" 'cargo test --all-features --all-targets --verbose'
Run-Step "Docs (deny doc warnings)" 'cargo doc --no-deps --all-features --verbose'

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
