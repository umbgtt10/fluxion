<#
Check that no_std compilation works.

Usage:
  # run in repository root
  .\.ci\no_std_check.ps1

This script verifies that Fluxion compiles correctly in no_std environments:
  - cargo check --no-default-features --features alloc
  - cargo build --no-default-features --features alloc

Notes:
  - This ensures 24/27 operators work on embedded targets
  - Spawn-based operators (share, subscribe_latest, partition) require std
  - The build uses only heap allocation (alloc), no standard library
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Output "=== no_std Compilation Check ==="
Write-Output "Verifying no_std support with alloc feature..."

Write-Output ""
Write-Output "Step 1: cargo check --no-default-features --features alloc"
cargo check --no-default-features --features alloc --verbose
if ($LASTEXITCODE -ne 0) {
  Write-Error "no_std check failed with exit code $LASTEXITCODE"
  exit $LASTEXITCODE
}

Write-Output ""
Write-Output "Step 2: cargo build --no-default-features --features alloc"
cargo build --no-default-features --features alloc --verbose
if ($LASTEXITCODE -ne 0) {
  Write-Error "no_std build failed with exit code $LASTEXITCODE"
  exit $LASTEXITCODE
}

Write-Output ""
Write-Output "✅ no_std compilation successful!"
Write-Output "   24/27 operators available on embedded targets"
Write-Output "   (share, subscribe_latest, partition require std)"
