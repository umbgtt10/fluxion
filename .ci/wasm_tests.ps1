<#
Run WASM tests for fluxion-stream-time (PowerShell).

Usage:
  # run from repository root
  .\.ci\wasm_tests.ps1

This runs:
  - wasm-pack test --node (WASM tests with Node.js runtime)

Prerequisites:
  - Node.js must be installed
  - wasm-pack will be installed automatically if missing

Notes:
  - Tests run in fluxion-stream-time crate only
  - Uses Node.js runtime (faster and more reliable than browsers for CI)
  - Requires time-wasm feature flag
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

Write-Output "Starting WASM tests..."

# Check if Node.js is available
try {
  $nodeVersion = & node --version 2>&1
  Write-Output "Node.js detected: $nodeVersion"
} catch {
  Write-Error "Node.js not found. WASM tests require Node.js to be installed."
  Write-Error "Install Node.js from https://nodejs.org/"
  exit 1
}

# Ensure wasm-pack is installed
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Output "wasm-pack not found; installing..."
    & cargo install --locked wasm-pack
    if ($LASTEXITCODE -ne 0) {
      Write-Error "Failed to install wasm-pack"
      exit $LASTEXITCODE
    }
}

# Run WASM tests from fluxion-stream-time directory
Push-Location fluxion-stream-time
try {
  Write-Output "=== Run WASM tests (Node.js runtime) ==="
  Write-Output "Note: Doc tests reference TokioTimer and will fail on wasm32 (expected)"
  Write-Output "Doc tests are validated in tokio_tests.ps1 for native/Tokio usage"
  Write-Output ""
  
  # Temporarily disable error action to capture output even if wasm-pack returns non-zero
  $previousErrorAction = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  
  # Run wasm-pack test - doc tests will fail (expected), but we check that WASM tests pass
  $output = & wasm-pack test --node --features time-wasm 2>&1 | Out-String
  $exitCode = $LASTEXITCODE
  
  $ErrorActionPreference = $previousErrorAction
  
  Write-Output $output
  
  # Check if actual WASM tests passed (tests/all_tests.rs)
  # Look for the line "test result: ok. 1 passed; 0 failed" from all_tests.rs
  if ($output -match "test result: ok\. 1 passed; 0 failed; 0 ignored") {
    Write-Output ""
    Write-Output "SUCCESS: WASM tests passed (doc test failures are expected and ignored)"
  } else {
    Write-Error "WASM tests did not pass. Check output above."
    exit 1
  }
} finally {
  Pop-Location
}

Write-Output "WASM tests completed successfully."
exit 0
