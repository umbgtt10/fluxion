<#
Run Embassy tests for fluxion-stream-time (PowerShell).

Usage:
  # run from repository root
  .\.ci\embassy_tests.ps1

This runs:
  - cargo +nightly test --package fluxion-stream-time --features runtime-embassy --no-default-features --test all_tests

Prerequisites:
  - Nightly Rust toolchain (embassy-executor tests require nightly)
  - embassy-time and embassy-executor dev-dependencies resolved automatically

Notes:
  - Tests run in fluxion-stream-time crate only
  - Uses Embassy executor with #[embassy_executor::test] attribute (requires nightly)
  - Embassy timer with std driver (generic-queue-8 feature)
  - Requires runtime-embassy feature flag
  - Tests validate authentic Embassy runtime behavior (no Tokio harness)
  - Library code remains stable-compatible; only tests use nightly
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

Write-Output "Starting Embassy tests (uses Embassy executor with tasks)..."

# Check if nightly toolchain is available
Write-Output "Checking for nightly toolchain..."
$nightlyCheck = cargo +nightly --version 2>&1
if ($LASTEXITCODE -ne 0) {
  Write-Warning "Nightly Rust toolchain not found. Embassy tests require nightly for embassy_executor::task macro."
  Write-Output "Install with: rustup toolchain install nightly"
  Write-Output "Skipping Embassy tests."
  exit 0
}

Invoke-StepAction "Run Embassy tests (fluxion-core)" {
  cargo +nightly test --package fluxion-core --features runtime-embassy --test all_tests -- --test-threads=1
}

Invoke-StepAction "Run Embassy tests (fluxion-stream)" {
  cargo +nightly test --package fluxion-stream --features runtime-embassy --test all_tests -- --test-threads=1
}

Invoke-StepAction "Run Embassy tests with executor (fluxion-stream-time)" {
  cargo +nightly test --package fluxion-stream-time --features runtime-embassy --no-default-features --test all_tests -- --test-threads=1
}

Invoke-StepAction "Build fluxion-runtime for Embassy (compilation check)" {
  cargo +nightly build --package fluxion-runtime --no-default-features --features runtime-embassy
}

Write-Output "Embassy tests completed successfully."
exit 0
