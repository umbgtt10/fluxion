#!/usr/bin/env pwsh
# Sync README examples from test files
# This ensures the README examples are always the actual tested code

$ErrorActionPreference = "Stop"

$example1Path = "fluxion/tests/example1_functional.rs"
$example2Path = "fluxion/tests/example2_composition.rs"
$readmePath = "README.md"
$cargoTomlPath = "Cargo.toml"

Write-Host "Syncing README examples from test files..." -ForegroundColor Cyan

# Read the test files
$example1Lines = Get-Content $example1Path
$example2Lines = Get-Content $example2Path

# Remove copyright header (first 4 lines) and join
$example1Code = ($example1Lines | Select-Object -Skip 4) -join "`n"
$example2Code = ($example2Lines | Select-Object -Skip 4) -join "`n"

# Add final newline
$example1Code = $example1Code + "`n"
$example2Code = $example2Code + "`n"

# Parse Cargo.toml to extract dependency versions
Write-Host "Extracting dependency versions from Cargo.toml..." -ForegroundColor Cyan
$cargoToml = Get-Content $cargoTomlPath -Raw

# Detect which external crates are used in the examples
$allExampleCode = $example1Code + $example2Code
$usedCrates = @{}

# Check for common external dependencies (both use statements and attribute/qualified usage)
$externalCrates = @('tokio', 'futures', 'anyhow', 'serde', 'tracing', 'async-trait')

foreach ($crate in $externalCrates) {
    $found = $false

    # Check for explicit use statements: use tokio::*, use futures::*
    if ($allExampleCode -match "use\s+$crate(::|\s|;)") {
        $found = $true
    }

    # Check for attributes: #[tokio::test], #[tokio::main]
    if ($allExampleCode -match "#\[\s*$crate::") {
        $found = $true
    }

    # Check for qualified usage: anyhow::Result, tokio::sync
    if ($allExampleCode -match "$crate::") {
        $found = $true
    }

    if ($found) {
        # Extract version from Cargo.toml
        if ($cargoToml -match "$crate\s*=\s*\{\s*version\s*=\s*`"([^`"]+)`"") {
            $usedCrates[$crate] = $matches[1]
        }
        elseif ($cargoToml -match "$crate\s*=\s*`"([^`"]+)`"") {
            $usedCrates[$crate] = $matches[1]
        }
    }
}

# Always include fluxion crates
$dependenciesLines = @(
    "[dependencies]",
    "fluxion-rx = `"0.2.0`"",
    "fluxion-test-utils = `"0.2.0`""
)

# Add tokio with features if used
if ($usedCrates.ContainsKey('tokio')) {
    $dependenciesLines += "tokio = { version = `"$($usedCrates['tokio'])`", features = [`"full`"] }"
}

# Add other dependencies
foreach ($crate in $usedCrates.Keys | Where-Object { $_ -ne 'tokio' } | Sort-Object) {
    $dependenciesLines += "$crate = `"$($usedCrates[$crate])`""
}

# Build dependencies section
$dependenciesSection = "``````toml`n" + ($dependenciesLines -join "`n") + "`n``````"

# Read README
$readme = Get-Content $readmePath -Raw

# Find the positions of the dependencies section
$quickStartHeader = $readme.IndexOf('## Quick Start')
$dependenciesStart = $readme.IndexOf('```toml', $quickStartHeader)
$dependenciesEnd = $readme.IndexOf('```', $dependenciesStart + 7)

# Find the positions of the code blocks
$basicUsageStart = $readme.IndexOf('### Basic Usage')
$basicUsageCodeStart = $readme.IndexOf('```rust', $basicUsageStart)
$basicUsageCodeEnd = $readme.IndexOf('```', $basicUsageCodeStart + 7)

$chainingStart = $readme.IndexOf('### Chaining Multiple Operators')
$chainingCodeStart = $readme.IndexOf('```rust', $chainingStart)
$chainingCodeEnd = $readme.IndexOf('```', $chainingCodeStart + 7)

# Build new README with updated dependencies and examples
$newReadme = $readme.Substring(0, $dependenciesStart) # Up to dependencies code block
$newReadme += $dependenciesSection
$newReadme += $readme.Substring($dependenciesEnd + 3, $basicUsageCodeStart - ($dependenciesEnd + 3) + 8) # From deps ``` to Basic Usage ```rust\n
$newReadme += $example1Code
$newReadme += $readme.Substring($basicUsageCodeEnd, $chainingCodeStart - $basicUsageCodeEnd + 8) # From first ``` to second ```rust\n
$newReadme += $example2Code
$newReadme += $readme.Substring($chainingCodeEnd) # From second ``` to end

# Write back
Set-Content $readmePath $newReadme -NoNewline

Write-Host "✓ README synced successfully!" -ForegroundColor Green
Write-Host "  - Dependencies extracted from Cargo.toml" -ForegroundColor Gray
Write-Host "  - example1_functional.rs → ### Basic Usage" -ForegroundColor Gray
Write-Host "  - example2_composition.rs → ### Chaining Multiple Operators" -ForegroundColor Gray
