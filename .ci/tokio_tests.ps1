<#
Run Tokio-based tests for the workspace (PowerShell).

Usage:
  # run from repository root
  .\.ci\tokio_tests.ps1

This runs:
  - cargo nextest run (all native tests with Tokio runtime)
  - cargo test --doc (doc tests)

Notes:
  - Installs cargo-nextest if not found
  - Embassy tests excluded from --all-features runs (require --no-default-features)
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-StepAction {
  param(
    [string]$Name,
    [ScriptBlock]$Action
  )

  Write-Output "=== $Name ==="
  & $Action
  $code = $LASTEXITCODE
  if ($code -ne 0) {
    Write-Error "Step '$Name' failed with exit code $code"
    exit $code
  }
}

Write-Output "Starting Tokio tests..."

# Ensure cargo-nextest is installed
if (-not (Get-Command cargo-nextest -ErrorAction SilentlyContinue)) {
    Write-Output "cargo-nextest not found; installing..."
    & cargo install --locked cargo-nextest
    if ($LASTEXITCODE -ne 0) {
      Write-Error "Failed to install cargo-nextest"
      exit $LASTEXITCODE
    }
}

Invoke-StepAction "Run nextest (Tokio runtime tests)" {
  # Exclude embassy tests which require --no-default-features and nightly features
  # Exclude wasm-dashboard which requires wasm32 target
  cargo nextest run --verbose --lib --bins --tests --examples --workspace --exclude wasm-dashboard
}

Invoke-StepAction "Run doc tests" {
  cargo test --doc --verbose --workspace --exclude wasm-dashboard
}

Write-Output "Tokio tests completed successfully."
