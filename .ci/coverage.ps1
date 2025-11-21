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
  Write-Output "cargo-llvm-cov not found; installing..."
  & cargo install cargo-llvm-cov
}

$outDir = Join-Path 'target' 'coverage'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Output "Running cargo llvm-cov (workspace) - lcov..."
Write-Output "Ensuring llvm-tools-preview component is available..."
$active = (& rustup show active-toolchain 2>$null) -split ' ' | Select-Object -First 1
if (-not $active) { $active = 'stable' }
Write-Output "Adding llvm-tools-preview to toolchain: $active"
& rustup component add llvm-tools-preview --toolchain $active

& cargo llvm-cov --workspace --exclude stream-aggregation --lcov --output-path "$outDir\lcov.info"

Write-Output "Running cargo llvm-cov (workspace) - html..."
& cargo llvm-cov --workspace --exclude stream-aggregation --html --output-dir "$outDir\html"

if ([int]$failUnder -gt 0) {
  Write-Output "Enforcing coverage threshold: $failUnder%"
  & cargo llvm-cov --fail-under $failUnder --workspace
}

Write-Output "Coverage artifacts written to $outDir"
