param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# Coverage runner using cargo-tarpaulin
# Produces HTML in target/coverage/html and lcov in target/coverage/lcov.info

$root = Join-Path $PSScriptRoot '..'
Set-Location $root

$failUnder = $env:COVERAGE_FAIL_UNDER
if (-not $failUnder) { $failUnder = 0 }

if (-not (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue)) {
  Write-Output "cargo-tarpaulin not found; installing..."
  & cargo install cargo-tarpaulin
}

$outDir = Join-Path 'target' 'coverage'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Output "Running cargo tarpaulin (workspace, excluding examples)..."

$tarpaulinArgs = @(
  'tarpaulin'
  '--workspace'
  '--exclude', 'stream-aggregation'
  '--out', 'Html'
  '--out', 'Lcov'
  '--output-dir', $outDir
)

if ([int]$failUnder -gt 0) {
  Write-Output "Enforcing coverage threshold: $failUnder%"
  $tarpaulinArgs += '--fail-under', $failUnder
}

& cargo @tarpaulinArgs

Write-Output "Coverage artifacts written to $outDir"
