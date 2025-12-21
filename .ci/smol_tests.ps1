<#
Run smol tests for fluxion-stream-time (PowerShell).

Usage:
  # run from repository root
  .\.ci\smol_tests.ps1

This runs:
  - cargo test --package fluxion-stream-time --features time-smol --no-default-features --test all_tests

Prerequisites:
  - smol dev-dependency will be resolved automatically

Notes:
  - Tests run in fluxion-stream-time crate only
  - Uses smol runtime (supports both single and multi-threaded execution)
  - Requires time-smol feature flag
  - Tests both single-threaded and multi-threaded scenarios
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

Write-Output "Starting smol tests..."

Invoke-StepAction "Run smol tests (fluxion-core)" {
  cargo test --package fluxion-core --features runtime-smol --no-default-features --test all_tests --verbose
}

Invoke-StepAction "Run smol tests (fluxion-stream-time)" {
  cargo test --package fluxion-stream-time --features runtime-smol --no-default-features --test all_tests --verbose
}

Write-Output "smol tests completed successfully."
exit 0
