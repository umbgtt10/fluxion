<#
Run the project's CI checks locally (PowerShell).

Usage:
  # run in repository root
  .\.ci\ci.ps1

This mirrors the GitHub Actions `ci.yml` steps:
  - cargo fmt --check
  - cargo check (all targets & features)
  - cargo clippy (deny warnings)
  - cargo build --release
  - .\.ci\build.ps1 (upgrade, build, test)
    - .\.ci\tokio_tests.ps1 (Tokio tests with nextest)
    - .\.ci\wasm_tests.ps1 (WASM tests with wasm-pack)
    - .\.ci\async_std_tests.ps1 (async-std tests)    - .ci\smol_tests.ps1 (smol tests)  - cargo doc --no-deps
  - cargo-audit (install if missing)

Notes:
  - This script runs on Windows PowerShell. On Unix/macOS use `.ci/run-ci.sh`.
  - Some commands (clippy with -D warnings) will fail the run if any warnings remain.
  - Running with `--all-features` may take longer but reduces false-positives.
  - WASM tests require Node.js to be installed.
  - All test scripts (.ci\tokio_tests.ps1, .ci\wasm_tests.ps1, .ci\async_std_tests.ps1, .ci\smol_tests.ps1) can be run standalone.
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

Write-Output "Starting local CI checks..."

Invoke-StepAction "Format check" { cargo fmt --all -- --check }

# Run upgrade & build early to fail fast on dependency or build regressions
Write-Output "=== Upgrade & build ==="
& .\.ci\build.ps1
$rc = $LASTEXITCODE
if ($rc -ne 0) {
  Write-Error "Upgrade & build failed (exit code $rc). Aborting CI. See .ci\\build.ps1 output for details."
  exit $rc
}

Invoke-StepAction "Cargo check (all targets & features)" { cargo check --all-targets --all-features --verbose }
Invoke-StepAction "Clippy (deny warnings)" { cargo clippy --all-targets --all-features -- -D warnings }
Invoke-StepAction "Release build" { cargo build --release --all-targets --all-features --verbose }
Invoke-StepAction "Benchmark compilation" { cargo bench --no-run --all-features --verbose }
Invoke-StepAction "Docs (deny doc warnings)" { cargo doc --no-deps --all-features --verbose }

Invoke-StepAction "Install nightly toolchain" { rustup toolchain install nightly }

# Ensure cargo-udeps is installed (needed for workspace-wide unused-deps analysis)
if (-not (Get-Command cargo-udeps -ErrorAction SilentlyContinue)) {
  Write-Output "cargo-udeps not found; installing..."
  & cargo install --locked cargo-udeps
  if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to install cargo-udeps"
    exit $LASTEXITCODE
  }
}

Write-Output "=== Run cargo +nightly udeps (check for unused deps) ==="
$stdoutFile = [System.IO.Path]::GetTempFileName()
$stderrFile = [System.IO.Path]::GetTempFileName()
$cargoArgs = @('+nightly','udeps','--all-targets','--all-features','--workspace')
$proc = Start-Process -FilePath 'cargo' -ArgumentList $cargoArgs -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile

$stdOut = Get-Content -Raw -Path $stdoutFile -ErrorAction SilentlyContinue
$stdErr = Get-Content -Raw -Path $stderrFile -ErrorAction SilentlyContinue
Remove-Item $stdoutFile, $stderrFile -ErrorAction SilentlyContinue

$udepsOutput = $stdOut + "`n" + $stdErr
Write-Output $udepsOutput

if ($proc.ExitCode -ne 0) {
    Write-Error "cargo-udeps failed with exit code $($proc.ExitCode)"
    exit $proc.ExitCode
}

if ($udepsOutput -notmatch 'All deps seem to have been used\.') {
  Write-Error "cargo-udeps reported unused dependencies; failing local CI."
  exit 1
}

Write-Output "=== Install & run cargo-audit ==="
try {
  cargo audit --version | Out-Null
  Write-Output "cargo-audit is installed"
} catch {
  Write-Output "cargo-audit not found; installing..."
  & cargo install --locked cargo-audit
  if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to install cargo-audit"
    exit $LASTEXITCODE
  }
}

Invoke-StepAction "cargo audit" { cargo audit }

Write-Output "All CI checks completed."
