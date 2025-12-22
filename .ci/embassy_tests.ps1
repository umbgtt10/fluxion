<#
Run Embassy tests for fluxion-stream-time (PowerShell).

Usage:
  # run from repository root
  .\.ci\embassy_tests.ps1

This runs:
  - cargo test --package fluxion-stream-time --features runtime-embassy --no-default-features --test all_tests

Prerequisites:
  - embassy-time and embassy-executor dev-dependencies will be resolved automatically

Notes:
  - Tests run in fluxion-stream-time crate only
  - Uses Embassy timer with std driver (generic-queue-8 feature)
  - Requires runtime-embassy feature flag
  - Tests validate Embassy timer integration (runs within Tokio executor as test harness)
  - Embassy's actual executor requires nightly Rust features
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

Write-Output "Starting Embassy tests..."

Invoke-StepAction "Run Embassy tests (fluxion-stream-time)" {
  cargo test --package fluxion-stream-time --features runtime-embassy --no-default-features --test all_tests -- --test-threads=1
}

Write-Output "Embassy tests completed successfully."
exit 0
