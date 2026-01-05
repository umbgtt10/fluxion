#!/usr/bin/env pwsh

# Change to project root
Set-Location (Join-Path $PSScriptRoot "..")

Write-Host "Building embassy-sensors for ARM Cortex-M4F..." -ForegroundColor Cyan
cargo build --release --target thumbv7em-none-eabihf

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Launching QEMU..." -ForegroundColor Green

# Use full path to QEMU (scoop installation)
$qemuPath = "$env:USERPROFILE\scoop\apps\qemu\current\qemu-system-arm.exe"
if (-not (Test-Path $qemuPath)) {
    # Try system PATH
    $qemuPath = "qemu-system-arm"
}

& $qemuPath `
    -cpu cortex-m4 `
    -machine mps2-an386 `
    -nographic `
    -semihosting-config enable=on,target=native `
    -kernel target/thumbv7em-none-eabihf/release/embassy-sensors

Write-Host "QEMU session ended" -ForegroundColor Yellow
