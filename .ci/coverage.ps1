param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Coverage runner using cargo-llvm-cov
# Produces HTML in target/coverage/html and lcov in target/coverage/lcov.info

$root = Join-Path $PSScriptRoot '..'
Set-Location $root

$failUnder = $env:COVERAGE_FAIL_UNDER
if (-not $failUnder) { $failUnder = 0 }

if (-not (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue)) {
  Write-Host "cargo-llvm-cov not found; installing..." -ForegroundColor Yellow
  & cargo install cargo-llvm-cov
}

$outDir = Join-Path 'target' 'coverage'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Host "Running cargo llvm-cov (workspace) - lcov..." -ForegroundColor Cyan
Write-Host "Ensuring llvm-tools-preview component is available..." -ForegroundColor Cyan
$active = (& rustup show active-toolchain 2>$null) -split ' ' | Select-Object -First 1
if (-not $active) { $active = 'stable' }
Write-Host "Adding llvm-tools-preview to toolchain: $active" -ForegroundColor Cyan
& rustup component add llvm-tools-preview --toolchain $active

& cargo llvm-cov --workspace --lcov --output-path "$outDir\lcov.info"

Write-Host "Running cargo llvm-cov (workspace) - html..." -ForegroundColor Cyan
& cargo llvm-cov --workspace --html --output-dir "$outDir\html"

if ([int]$failUnder -gt 0) {
  Write-Host "Enforcing coverage threshold: $failUnder%" -ForegroundColor Yellow
  & cargo llvm-cov --fail-under $failUnder --workspace
}

Write-Host "Coverage artifacts written to $outDir" -ForegroundColor Green
