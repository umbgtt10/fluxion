#!/usr/bin/env pwsh
# Generate HTML coverage report using cargo-llvm-cov
# This script runs coverage with runtime-tokio feature to ensure
# fluxion-stream-time tests are properly executed (mutually exclusive runtime features)

Write-Host "Generating coverage report with HTML output..." -ForegroundColor Cyan

# Clean previous coverage data
cargo llvm-cov clean

# Run coverage with HTML output
cargo llvm-cov --workspace `
  --exclude fluxion-test-utils `
  --exclude wasm-dashboard `
  --exclude stream-aggregation `
  --exclude legacy-integration `
  --exclude embassy-sensors `
  --features "runtime-tokio" `
  --ignore-filename-regex "fluxion-test-utils" `
  --html

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Coverage report generated successfully!" -ForegroundColor Green
    Write-Host "View report at: target\llvm-cov\html\index.html" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To open in browser:" -ForegroundColor Cyan
    Write-Host "  Start-Process target\llvm-cov\html\index.html" -ForegroundColor White
} else {
    Write-Host ""
    Write-Host "[ERROR] Coverage generation failed" -ForegroundColor Red
    exit $LASTEXITCODE
}
