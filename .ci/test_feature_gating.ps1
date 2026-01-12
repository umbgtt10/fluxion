#!/usr/bin/env pwsh
# Test feature-gating to ensure operators are correctly included/excluded

param(
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

function Write-TestHeader {
    param([string]$Message)
    Write-Host "`n=== $Message ===" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[PASS] $Message" -ForegroundColor Green
}

function Write-Failure {
    param([string]$Message)
    Write-Host "[FAIL] $Message" -ForegroundColor Red
}

function Test-Compilation {
    param(
        [string]$Package,
        [string]$Features,
        [string]$Description
    )

    Write-Host "  Testing: $Description" -NoNewline

    $args = @("check", "--package", $Package, "--quiet")
    if ($Features) {
        $args += @("--no-default-features", "--features", $Features)
    }

    $output = & cargo $args 2>&1
    $success = $LASTEXITCODE -eq 0

    if ($success) {
        Write-Host " - " -NoNewline
        Write-Success "Compiles"
        return $true
    } else {
        Write-Host ""
        Write-Failure "Failed to compile"
        if ($Verbose) {
            Write-Host $output -ForegroundColor Yellow
        }
        return $false
    }
}

function Test-SymbolPresence {
    param(
        [string]$Package,
        [string]$Features,
        [string[]]$ExpectedSymbols,
        [string[]]$UnexpectedSymbols,
        [string]$Description
    )

    Write-Host "  Testing: $Description"

    # Clean previous docs to ensure fresh build (package names use underscores)
    $docPackageName = $Package -replace '-', '_'
    $docPath = "target/doc/$docPackageName"
    if (Test-Path $docPath) {
        Remove-Item -Recurse -Force $docPath 2>&1 | Out-Null
    }

    # Build with specific features to generate docs
    $args = @("doc", "--package", $Package, "--no-deps", "--quiet")
    if ($Features) {
        $args += @("--no-default-features", "--features", $Features)
    }

    # Temporarily allow errors to continue (doc warnings shouldn't stop the script)
    $prevErrorAction = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $null = & cargo $args 2>&1
    $ErrorActionPreference = $prevErrorAction

    # Check if docs were actually generated
    if (-not (Test-Path $docPath)) {
        Write-Failure "  Failed to generate docs (no doc directory found at $docPath)"
        return $false
    }

    $allPassed = $true

    # Check expected symbols exist (look for re-export declarations OR trait/struct/type definitions)
    foreach ($symbol in $ExpectedSymbols) {
        # Pattern matches: id=.reexport.Symbol OR href="trait.Symbol.html" OR href="struct.Symbol.html" OR href="type.Symbol.html"
        $pattern = "(id=.reexport\.$symbol|href=.(trait|struct|type|enum)\.$symbol\.html)"
        $found = Select-String -Path "target/doc/$docPackageName/*.html" -Pattern $pattern -Quiet 2>$null
        if ($found) {
            Write-Success "    Found expected: $symbol"
            $script:SuccessCount++
        } else {
            Write-Failure "    Missing expected: $symbol"
            $script:FailureCount++
            $allPassed = $false
        }
    }

    # Check unexpected symbols don't exist (look for re-export declarations OR trait/struct/type definitions)
    foreach ($symbol in $UnexpectedSymbols) {
        $pattern = "(id=.reexport\.$symbol|href=.(trait|struct|type|enum)\.$symbol\.html)"
        $found = Select-String -Path "target/doc/$docPackageName/*.html" -Pattern $pattern -Quiet 2>$null
        if (-not $found) {
            Write-Success "    Correctly excluded: $symbol"
            $script:SuccessCount++
        } else {
            Write-Failure "    Should not have: $symbol"
            $script:FailureCount++
            $allPassed = $false
        }
    }

    return $allPassed
}

# Track results
$script:FailureCount = 0
$script:SuccessCount = 0

function Record-Result {
    param([bool]$Success)
    if ($Success) {
        $script:SuccessCount++
    } else {
        $script:FailureCount++
    }
}

# ============================================================================
# Comprehensive operator lists
# ============================================================================

# All fluxion-stream operators/types available in no_std + alloc
# 22 stream operators (20 non-gated + utility trait) + 3 types = 23 items
$AllNonGatedStreamOperators = @(
    # Stream Operators (22 non-gated)
    "CombineLatestExt",
    "CombineWithPreviousExt",
    "DistinctUntilChangedExt",
    "DistinctUntilChangedByExt",
    "EmitWhenExt",
    "FilterOrderedExt",
    "IntoFluxionStream",
    "MapOrderedExt",
    "OnErrorExt",
    "OrderedStreamExt",
    "SampleRatioExt",
    "ScanOrderedExt",
    "SkipItemsExt",
    "StartWithExt",
    "TakeItemsExt",
    "TakeLatestWhenExt",
    "TapExt",
    "TakeWhileExt",
    "WindowByCountExt",
    "WithLatestFromExt",
    # Types (3 non-gated)
    "CombinedState",
    "MergedStream",
    "WithPrevious"
)

# Multi-threaded runtime-gated fluxion-stream operators/types (2 operators + 2 types = 4 items)
# Available on: Tokio, smol, async-std, WASM
# NOT available on: Embassy (single-threaded, no spawning)
$MultiThreadedStreamOperators = @(
    "PartitionExt",
    "ShareExt",
    # Types
    "FluxionShared",
    "PartitionedStream"
)

# All fluxion-stream operators/types with multi-threaded runtime
# 20+2=22 stream operators + 3+2=5 types = 27 stream items total
$AllStreamOperators = $AllNonGatedStreamOperators + $MultiThreadedStreamOperators

# fluxion-stream-time operators (5 time operators + 1 type)
# Available on ALL runtimes (Tokio, smol, async-std, WASM, Embassy)
$AllTimeOperators = @(
    "DebounceExt",
    "ThrottleExt",
    "DelayExt",
    "SampleExt",
    "TimeoutExt",
    "InstantTimestamped"
)

# fluxion-exec operators
$NonGatedExecOperators = @(
    "SubscribeExt"
)

# Multi-threaded runtime-gated (NOT available on Embassy)
# Available on: Tokio, smol, async-std, WASM
$MultiThreadedExecOperators = @(
    "SubscribeLatestExt"
)

$AllExecOperators = $NonGatedExecOperators + $MultiThreadedExecOperators

# ============================================================================
# Test 1: Compilation - All 5 Runtimes
# ============================================================================
Write-TestHeader "Test 1: Compilation - All 5 Runtimes"

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-tokio" -Description "Tokio runtime"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-smol" -Description "smol runtime"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-async-std" -Description "async-std runtime"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-embassy" -Description "Embassy runtime"
Record-Result $result

Write-Host "  Note: WASM runtime requires wasm32 target - tested separately in CI"

# ============================================================================
# Test 2: fluxion-stream - Multi-threaded Runtimes (Tokio, smol, async-std)
# ============================================================================
Write-TestHeader "Test 2: fluxion-stream - Multi-threaded Runtimes"
Write-Host "  Testing Tokio runtime..."
Write-Host "  Expected: All $($AllStreamOperators.Count) items present (22 stream operators + 5 types)"
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "runtime-tokio" `
    -ExpectedSymbols $AllStreamOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio: All operators including partition/share present"
Record-Result $result

Write-Host "  Testing smol runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "runtime-smol" `
    -ExpectedSymbols $AllStreamOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-smol: All operators including partition/share present"
Record-Result $result

Write-Host "  Testing async-std runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "runtime-async-std" `
    -ExpectedSymbols $AllStreamOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-async-std: All operators including partition/share present"
Record-Result $result

# ============================================================================
# Test 3: fluxion-stream - Embassy Runtime (Single-threaded)
# ============================================================================
Write-TestHeader "Test 3: fluxion-stream - Embassy Runtime"

Write-Host "  Checking symbol presence for Embassy..."
Write-Host "  Expected: $($AllNonGatedStreamOperators.Count) stream operators present, $($MultiThreadedStreamOperators.Count) multi-threaded operators absent"
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "runtime-embassy" `
    -ExpectedSymbols $AllNonGatedStreamOperators `
    -UnexpectedSymbols $MultiThreadedStreamOperators `
    -Description "runtime-embassy: Base operators present, partition/share absent (no spawning)"
Record-Result $result

# ============================================================================
# Test 4: fluxion-exec - Multi-threaded Runtimes (Tokio, smol, async-std)
# ============================================================================
Write-TestHeader "Test 4: fluxion-exec - Multi-threaded Runtimes"

Write-Host "  Testing Tokio runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "runtime-tokio" `
    -ExpectedSymbols $AllExecOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio: Both subscribe and subscribe_latest present"
Record-Result $result

Write-Host "  Testing smol runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "runtime-smol" `
    -ExpectedSymbols $AllExecOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-smol: Both subscribe and subscribe_latest present"
Record-Result $result

Write-Host "  Testing async-std runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "runtime-async-std" `
    -ExpectedSymbols $AllExecOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-async-std: Both subscribe and subscribe_latest present"
Record-Result $result

# ============================================================================
# Test 5: fluxion-exec - Embassy (Only subscribe, no subscribe_latest)
# ============================================================================
Write-TestHeader "Test 5: fluxion-exec - Embassy Runtime"

Write-Host "  Note: fluxion-exec does not have runtime-embassy feature"
Write-Host "  Embassy only supports subscribe (no subscribe_latest - no spawning)"

# ============================================================================
# Test 6: fluxion-stream-time - All 5 Runtimes
# ============================================================================
Write-TestHeader "Test 6: fluxion-stream-time - All 5 Runtimes"

Write-Host "  Testing Tokio runtime..."
Write-Host "  Expected: All $($AllTimeOperators.Count) time operators present"
$result = Test-SymbolPresence `
    -Package "fluxion-stream-time" `
    -Features "runtime-tokio" `
    -ExpectedSymbols $AllTimeOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio: All time operators present"
Record-Result $result

Write-Host "  Testing smol runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream-time" `
    -Features "runtime-smol" `
    -ExpectedSymbols $AllTimeOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-smol: All time operators present"
Record-Result $result

Write-Host "  Testing async-std runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream-time" `
    -Features "runtime-async-std" `
    -ExpectedSymbols $AllTimeOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-async-std: All time operators present"
Record-Result $result

Write-Host "  Testing WASM runtime..."
Write-Host "  Note: WASM docs require wasm32 target - skipping on native platforms"
Write-Host "  (WASM operator availability verified in CI wasm-pack tests)"
# WASM doc generation requires wasm32-unknown-unknown target, skip for native CI

Write-Host "  Testing Embassy runtime..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream-time" `
    -Features "runtime-embassy" `
    -ExpectedSymbols $AllTimeOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-embassy: All time operators present (dual trait bound system)"
Record-Result $result

# ============================================================================
# Summary
# ============================================================================
Write-TestHeader "Test Summary"

$total = $script:SuccessCount + $script:FailureCount
Write-Host "`nTotal tests: $total" -ForegroundColor Cyan
Write-Host "Passed: $script:SuccessCount" -ForegroundColor Green
Write-Host "Failed: $script:FailureCount" -ForegroundColor $(if ($script:FailureCount -gt 0) { "Red" } else { "Green" })

if ($script:FailureCount -gt 0) {
    Write-Host "`nFeature gating tests FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "`nAll feature gating tests PASSED" -ForegroundColor Green
    exit 0
}
