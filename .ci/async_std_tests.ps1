<#
Run async-std tests for fluxion-stream-time (PowerShell).

Usage:
  # run from repository root
  .\.ci\async_std_tests.ps1

This runs:
  - cargo test --package fluxion-stream-time --features time-async-std --no-default-features --test all_tests

Prerequisites:
  - async-std dev-dependency will be resolved automatically

Notes:
  - Tests run in fluxion-stream-time crate only
  - Uses async-std runtime (multi-threaded by default)
  - Requires time-async-std feature flag
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

Write-Output "Starting async-std tests..."

Invoke-StepAction "Run async-std tests (fluxion-core)" {
  cargo test --package fluxion-core --features runtime-async-std --no-default-features --test all_tests --verbose
}

Invoke-StepAction "Run async-std tests (fluxion-stream-time)" {
  cargo test --package fluxion-stream-time --features runtime-async-std --no-default-features --test all_tests --verbose
}

Invoke-StepAction "Run async-std tests (fluxion-stream)" {
  cargo test --package fluxion-stream --features runtime-async-std --no-default-features --test all_tests --verbose
}

Invoke-StepAction "Run async-std tests (fluxion-exec)" {
  cargo test --package fluxion-exec --features runtime-async-std --no-default-features --verbose
}

Write-Output "async-std tests completed successfully."
exit 0
