<#
Check that no_std compilation works.

Usage:
  # run in repository root
  .\.ci\no_std_check.ps1

This script verifies that Fluxion compiles correctly in no_std environments:
  - Tests each library crate independently with no_std + alloc
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

# Test each library crate independently to catch always-on std dependencies
$cratesWithAllocFeature = @(
    "fluxion-core",
    "fluxion-stream",
    "fluxion-exec"
)

$cratesWithoutFeatures = @(
    "fluxion-ordered-merge"
)

Write-Output ""
Write-Output "Step 1: Testing each crate independently with no_std + alloc"
Write-Output "=========================================================="

foreach ($crate in $cratesWithAllocFeature) {
    Write-Output ""
    Write-Output "Testing: $crate"
    Write-Output "  Command: cargo check --no-default-features --features alloc -p $crate"
    cargo check --no-default-features --features alloc -p $crate
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAILED: no_std check failed for $crate with exit code $LASTEXITCODE"
        Write-Error "   This crate has always-on std dependencies!"
        exit $LASTEXITCODE
    }
    Write-Output "  OK: $crate no_std + alloc works"
}

foreach ($crate in $cratesWithoutFeatures) {
    Write-Output ""
    Write-Output "Testing: $crate (no features)"
    Write-Output "  Command: cargo check --no-default-features -p $crate"
    cargo check --no-default-features -p $crate
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAILED: no_std check failed for $crate with exit code $LASTEXITCODE"
        Write-Error "   This crate has always-on std dependencies!"
        exit $LASTEXITCODE
    }
    Write-Output "  OK: $crate no_std works"
}

Write-Output ""
Write-Output "Step 2: cargo check --no-default-features --features alloc (workspace)"
cargo check --no-default-features --features alloc --verbose
if ($LASTEXITCODE -ne 0) {
  Write-Error "no_std check failed with exit code $LASTEXITCODE"
  exit $LASTEXITCODE
}

Write-Output ""
Write-Output "Step 3: cargo build --no-default-features --features alloc (workspace)"
cargo build --no-default-features --features alloc --verbose
if ($LASTEXITCODE -ne 0) {
  Write-Error "no_std build failed with exit code $LASTEXITCODE"
  exit $LASTEXITCODE
}

Write-Output ""
Write-Output "Step 4: Embedded target compilation check (ARM Cortex-M4)"
Write-Output "=========================================================="

# Check if thumbv7em-none-eabihf target is installed
Write-Output "Checking for thumbv7em-none-eabihf target..."
$targetCheck = rustup target list | Select-String "thumbv7em-none-eabihf \(installed\)"
if (-not $targetCheck) {
    Write-Output "Installing thumbv7em-none-eabihf target..."
    rustup target add thumbv7em-none-eabihf
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install embedded target"
        exit $LASTEXITCODE
    }
}

# Test each crate on actual embedded target (ARM Cortex-M4F)
Write-Output ""
Write-Output "Testing crates on thumbv7em-none-eabihf (ARM Cortex-M4F)..."

foreach ($crate in $cratesWithAllocFeature) {
    Write-Output ""
    Write-Output "Testing: $crate on embedded target"
    Write-Output "  Command: cargo check --target thumbv7em-none-eabihf --no-default-features --features alloc -p $crate --lib"
    cargo check --target thumbv7em-none-eabihf --no-default-features --features alloc -p $crate --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAILED: Embedded target check failed for $crate"
        Write-Error "   This crate pulls in std dependencies!"
        exit $LASTEXITCODE
    }
    Write-Output "  OK: $crate works on embedded target"
}

foreach ($crate in $cratesWithoutFeatures) {
    Write-Output ""
    Write-Output "Testing: $crate on embedded target (no features)"
    Write-Output "  Command: cargo check --target thumbv7em-none-eabihf --no-default-features -p $crate --lib"
    cargo check --target thumbv7em-none-eabihf --no-default-features -p $crate --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Error "FAILED: Embedded target check failed for $crate"
        Write-Error "   This crate pulls in std dependencies!"
        exit $LASTEXITCODE
    }
    Write-Output "  OK: $crate works on embedded target"
}

Write-Output ""
Write-Output "SUCCESS: no_std compilation works!"
Write-Output "   All library crates support no_std + alloc on x86_64 AND ARM Cortex-M4"
Write-Output "   24/27 operators available on embedded targets"

