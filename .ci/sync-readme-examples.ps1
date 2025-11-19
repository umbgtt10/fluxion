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
    "fluxion-rx = `"0.2.1`"",
    "fluxion-test-utils = `"0.2.1`""
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

# Sync fluxion-exec README subscribe_async example
Write-Host "
Syncing fluxion-exec subscribe_async example..." -ForegroundColor Cyan

$subscribeAsyncExamplePath = "fluxion-exec/tests/subscribe_async_example.rs"

if (Test-Path $subscribeAsyncExamplePath) {
    $subscribeAsyncContent = Get-Content $subscribeAsyncExamplePath -Raw
    $subscribeAsyncLines = $subscribeAsyncContent -split "`r?`n"
    $subscribeAsyncCode = (($subscribeAsyncLines | Select-Object -Skip 4) -join "`n") + "`n"

    # Extract dependencies from the example file's use statements
    $usedExecCrates = @{}
    $exampleUseStatements = $subscribeAsyncLines | Where-Object { $_ -match '^use ' }

    # Always include fluxion-exec
    $usedExecCrates['fluxion-exec'] = '0.2.1'

    # Check for other crates in use statements
    $execExternalCrates = @('tokio', 'tokio-stream', 'tokio-util', 'anyhow', 'thiserror')
    foreach ($crate in $execExternalCrates) {
        # Convert hyphens to underscores for matching Rust import names
        $rustCrateName = $crate -replace '-', '_'
        if ($exampleUseStatements -match "use\s+$rustCrateName(::|\s|;)") {
            # Extract version from workspace Cargo.toml
            if ($cargoToml -match "$crate\s*=\s*\{\s*version\s*=\s*`"([^`"]+)`"") {
                $usedExecCrates[$crate] = $matches[1]
            }
            elseif ($cargoToml -match "$crate\s*=\s*`"([^`"]+)`"") {
                $usedExecCrates[$crate] = $matches[1]
            }
        }
    }

    # Build dependencies section for subscribe_async
    $execDepsLines = @("[dependencies]", "fluxion-exec = `"0.2.1`"")

    # Add tokio with features if used
    if ($usedExecCrates.ContainsKey('tokio')) {
        $execDepsLines += "tokio = { version = `"$($usedExecCrates['tokio'])`", features = [`"full`"] }"
    }

    # Add other dependencies
    foreach ($crate in $usedExecCrates.Keys | Where-Object { $_ -ne 'tokio' -and $_ -ne 'fluxion-exec' } | Sort-Object) {
        $execDepsLines += "$crate = `"$($usedExecCrates[$crate])`""
    }

    $execDepsSection = ($execDepsLines -join "`n")

    # Update main README with subscribe_async example
    $mainReadme = Get-Content $readmePath -Raw

    # Find the Sequential Processing section
    $sequentialStart = $mainReadme.IndexOf('**Sequential Processing:**')
    if ($sequentialStart -ge 0) {
        # Find the Dependencies section
        $depsLabel = $mainReadme.IndexOf('**Dependencies:**', $sequentialStart)
        $depsStart = $mainReadme.IndexOf('```toml', $depsLabel)
        $depsEnd = $mainReadme.IndexOf('```', $depsStart + 7)

        # Find the Example code block after Dependencies
        $exampleLabel = $mainReadme.IndexOf('**Example:**', $depsEnd)
        $exampleStart = $mainReadme.IndexOf('```rust', $exampleLabel)
        $exampleEnd = $mainReadme.IndexOf('```', $exampleStart + 7)

        if ($depsStart -ge 0 -and $depsEnd -ge 0 -and $exampleStart -ge 0 -and $exampleEnd -ge 0) {
            # Replace both dependencies and example code
            $newMainReadme = $mainReadme.Substring(0, $depsStart + 8) # Up to ```toml`n
            $newMainReadme += $execDepsSection + "`n"
            $newMainReadme += $mainReadme.Substring($depsEnd, $exampleStart - $depsEnd + 8) # From ```` to ```rust`n
            $newMainReadme += $subscribeAsyncCode
            $newMainReadme += $mainReadme.Substring($exampleEnd) # From ```` to end

            Set-Content $readmePath $newMainReadme -NoNewline
            Write-Host "   Updated subscribe_async dependencies and example in main README" -ForegroundColor Green
        }
    }
} else {
    Write-Host "   Warning: subscribe_async_example.rs not found" -ForegroundColor Yellow
}

# Sync subscribe_latest_async example
Write-Host "
Syncing fluxion-exec subscribe_latest_async example..." -ForegroundColor Cyan

$subscribeLatestAsyncExamplePath = "fluxion-exec/tests/subscribe_latest_async_example.rs"

if (Test-Path $subscribeLatestAsyncExamplePath) {
    $subscribeLatestAsyncContent = Get-Content $subscribeLatestAsyncExamplePath -Raw
    $subscribeLatestAsyncLines = $subscribeLatestAsyncContent -split "`r?`n"
    $subscribeLatestAsyncCode = (($subscribeLatestAsyncLines | Select-Object -Skip 4) -join "`n") + "`n"

    # Extract dependencies from the example file's use statements
    $usedLatestCrates = @{}
    $latestExampleUseStatements = $subscribeLatestAsyncLines | Where-Object { $_ -match '^use ' }

    # Always include fluxion-exec
    $usedLatestCrates['fluxion-exec'] = '0.2.1'

    # Check for other crates in use statements
    $latestExternalCrates = @('tokio', 'tokio-stream', 'tokio-util', 'anyhow', 'thiserror')
    foreach ($crate in $latestExternalCrates) {
        # Convert hyphens to underscores for matching Rust import names
        $rustCrateName = $crate -replace '-', '_'
        if ($latestExampleUseStatements -match "use\s+$rustCrateName(::|\s|;)") {
            # Extract version from workspace Cargo.toml
            if ($cargoToml -match "$crate\s*=\s*\{\s*version\s*=\s*`"([^`"]+)`"") {
                $usedLatestCrates[$crate] = $matches[1]
            }
            elseif ($cargoToml -match "$crate\s*=\s*`"([^`"]+)`"") {
                $usedLatestCrates[$crate] = $matches[1]
            }
        }
    }

    # Build dependencies section for subscribe_latest_async
    $latestDepsLines = @("[dependencies]", "fluxion-exec = `"0.2.1`"")

    # Add tokio with features if used
    if ($usedLatestCrates.ContainsKey('tokio')) {
        $latestDepsLines += "tokio = { version = `"$($usedLatestCrates['tokio'])`", features = [`"full`"] }"
    }

    # Add other dependencies
    foreach ($crate in $usedLatestCrates.Keys | Where-Object { $_ -ne 'tokio' -and $_ -ne 'fluxion-exec' } | Sort-Object) {
        $latestDepsLines += "$crate = `"$($usedLatestCrates[$crate])`""
    }

    $latestDepsSection = ($latestDepsLines -join "`n")

    # Update main README with subscribe_latest_async example
    $mainReadme = Get-Content $readmePath -Raw

    # Find the Latest-Value Processing section
    $latestValueStart = $mainReadme.IndexOf('**Latest-Value Processing (with auto-cancellation):**')
    if ($latestValueStart -ge 0) {
        # Find the Dependencies section
        $latestDepsLabel = $mainReadme.IndexOf('**Dependencies:**', $latestValueStart)
        $latestDepsStart = $mainReadme.IndexOf('```toml', $latestDepsLabel)
        $latestDepsEnd = $mainReadme.IndexOf('```', $latestDepsStart + 7)

        # Find the Example code block after Dependencies
        $latestExampleLabel = $mainReadme.IndexOf('**Example:**', $latestDepsEnd)
        $latestExampleStart = $mainReadme.IndexOf('```rust', $latestExampleLabel)
        $latestExampleEnd = $mainReadme.IndexOf('```', $latestExampleStart + 7)

        if ($latestDepsStart -ge 0 -and $latestDepsEnd -ge 0 -and $latestExampleStart -ge 0 -and $latestExampleEnd -ge 0) {
            # Replace both dependencies and example code
            $newMainReadme = $mainReadme.Substring(0, $latestDepsStart + 8) # Up to ```toml`n
            $newMainReadme += $latestDepsSection + "`n"
            $newMainReadme += $mainReadme.Substring($latestDepsEnd, $latestExampleStart - $latestDepsEnd + 8) # From ```` to ```rust`n
            $newMainReadme += $subscribeLatestAsyncCode
            $newMainReadme += $mainReadme.Substring($latestExampleEnd) # From ```` to end

            Set-Content $readmePath $newMainReadme -NoNewline
            Write-Host "   Updated subscribe_latest_async dependencies and example in main README" -ForegroundColor Green
        }
    }
} else {
    Write-Host "   Warning: subscribe_latest_async_example.rs not found" -ForegroundColor Yellow
}
