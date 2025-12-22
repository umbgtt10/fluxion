# Test no_std compilation on embedded target
# This script verifies that fluxion-core can compile for ARM Cortex-M targets

Write-Host "Testing fluxion-core compilation for embedded target thumbv7em-none-eabihf..." -ForegroundColor Cyan

# Check if the target is installed
$targetInstalled = rustup target list | Select-String "thumbv7em-none-eabihf \(installed\)"
if (-not $targetInstalled) {
    Write-Host "Installing thumbv7em-none-eabihf target..." -ForegroundColor Yellow
    rustup target add thumbv7em-none-eabihf
}

# Build for embedded target with no_std + alloc
Write-Host "`nBuilding fluxion-core with --no-default-features --features alloc..." -ForegroundColor Cyan
cargo build --target thumbv7em-none-eabihf --no-default-features --features alloc -p fluxion-core

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ SUCCESS: fluxion-core compiles for embedded target!" -ForegroundColor Green
    Write-Host "Target: thumbv7em-none-eabihf (ARM Cortex-M4F with hardware floating point)" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n❌ FAILED: Compilation errors detected" -ForegroundColor Red
    exit 1
}
