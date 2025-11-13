<#
Upgrade all workspace dependencies to the latest versions (including major bumps), then build and test.

Usage:
  # run in repository root
  powershell -NoProfile -ExecutionPolicy Bypass -File .\.ci\build.ps1

Behaviour:
  - Installs `cargo-edit` if missing (provides `cargo upgrade`).
  - Runs `cargo upgrade --workspace -a` to upgrade manifests to the newest versions.
  - Runs `cargo update` to refresh the lockfile.
  - Runs formatting check, build, and tests. Any failure aborts the script with a non-zero exit code.

Warning:
  This will modify `Cargo.toml` files. Run in a branch so you can review diffs and CI results.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Color([string]$Text, [ConsoleColor]$Color) {
  Write-Output $Text
}

function Invoke-StepAction {
  param(
    [string]$Name,
    [ScriptBlock]$Action
  )

  Write-Color "=== $Name ===" Yellow
  & $Action
  $code = $LASTEXITCODE
  if ($code -ne 0) {
    Write-Color "Step '$Name' failed with exit code $code" Red
    exit $code
  }
}

Write-Color "Starting upgrade + build + test sequence..." Cyan

# Ensure cargo-edit (cargo upgrade) is available
if (-not (Get-Command cargo-upgrade -ErrorAction SilentlyContinue)) {
    Write-Color "cargo-upgrade not found; installing cargo-edit (provides cargo upgrade)..." Cyan
    & cargo install --locked cargo-edit
    if ($LASTEXITCODE -ne 0) {
      Write-Color "Failed to install cargo-edit" Red
      exit $LASTEXITCODE
    }
}

 # Attempt workspace upgrade with fallback
function Invoke-WorkspaceUpgrade {
  Write-Color "Checking for cargo-edit (cargo upgrade) availability..." Cyan

  # Try to get help output from cargo upgrade. If it fails, cargo-edit is missing.
  $helpOutput = & cargo upgrade --help 2>&1 | Out-String
  $helpRc = $LASTEXITCODE

  if ($helpRc -ne 0) {
    Write-Color "cargo upgrade not found. Installing cargo-edit (cargo upgrade)..." Yellow
    & cargo install --locked cargo-edit --force
    $installRc = $LASTEXITCODE
    if ($installRc -ne 0) {
      Write-Color "Failed to install cargo-edit (exit code $installRc). Will proceed with per-crate compatible upgrades using whatever is available." Red
      $helpOutput = ""
    } else {
      # re-fetch help output after install
      $helpOutput = & cargo upgrade --help 2>&1 | Out-String
      $helpRc = $LASTEXITCODE
    }
  }

  # Detect supported flags from help text
  $hasWorkspace = $false
  $hasAggressive = $false
  if ($helpOutput) {
    if ($helpOutput -match '--workspace') { $hasWorkspace = $true }
    if ($helpOutput -match '--aggressive' -or $helpOutput -match '\-a[,\s]') { $hasAggressive = $true }
  }

  if ($hasWorkspace -and $hasAggressive) {
    Write-Color "Running workspace-wide aggressive upgrade (major bumps allowed)..." Cyan
    & cargo upgrade --workspace -a
    $rc = $LASTEXITCODE
    if ($rc -eq 0) { Write-Color "Workspace-wide aggressive upgrade succeeded." Green; return 0 }
    Write-Color "Workspace-wide aggressive upgrade failed (exit code $rc)." Yellow
  }

  if ($hasWorkspace) {
    Write-Color "Running workspace-wide compatible upgrade..." Cyan
    & cargo upgrade --workspace
    $rc2 = $LASTEXITCODE
    if ($rc2 -eq 0) { Write-Color "Workspace-wide compatible upgrade succeeded." Green; return 0 }
    Write-Color "Workspace-wide compatible upgrade failed (exit code $rc2)." Yellow
  }

  Write-Color "Falling back to per-crate upgrades (will respect available cargo-edit flags)..." Cyan

  # Obtain workspace members from cargo metadata
  $metaRaw = & cargo metadata --format-version 1 --no-deps 2>&1 | Out-String
  try {
    $meta = $metaRaw | ConvertFrom-Json
  } catch {
    Write-Color "Failed to parse cargo metadata output:" Red
    Write-Output $metaRaw
    exit 1
  }

  $memberIds = $meta.workspace_members
  foreach ($memberId in $memberIds) {
    $pkg = $meta.packages | Where-Object { $_.id -eq $memberId }
    if (-not $pkg) { continue }
    $manifest = $pkg.manifest_path
    $dir = Split-Path $manifest
    Write-Color "Upgrading manifest in $dir" Cyan
    Push-Location $dir

    if ($hasAggressive) {
      & cargo upgrade -a
      $rc = $LASTEXITCODE
      if ($rc -ne 0) {
        Write-Color "Per-crate aggressive upgrade failed in $dir; trying compatible-only upgrade..." Yellow
        & cargo upgrade
        $rc = $LASTEXITCODE
      }
    } else {
      & cargo upgrade
      $rc = $LASTEXITCODE
    }

    Pop-Location
    if ($rc -ne 0) {
      Write-Color "cargo upgrade failed in $dir with exit code $rc" Red
      exit $rc
    }
  }

  Write-Color "Per-crate upgrades completed." Green
  return 0
}

Invoke-WorkspaceUpgrade
Invoke-StepAction "Refresh lockfile" { cargo update }

Invoke-StepAction "Format check" { cargo fmt --all -- --check }
Invoke-StepAction "Build (all targets & features)" { cargo build --all-targets --all-features --verbose }
Invoke-StepAction "Clippy (deny warnings)" { cargo clippy --all-targets --all-features -- -D warnings }
Invoke-StepAction "Run tests" { cargo test --all-features --all-targets --verbose }

Write-Color "Upgrade + build + test sequence completed successfully." Green
