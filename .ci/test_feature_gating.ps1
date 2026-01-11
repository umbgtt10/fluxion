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

    # Check expected symbols exist (look for trait/struct/type definitions or reexports)
    foreach ($symbol in $ExpectedSymbols) {
        # Look for: trait links, struct links, type links, or reexport declarations
        $patterns = @(
            "trait\.$symbol",
            "struct\.$symbol",
            "type\.$symbol",
            "reexport\.$symbol"
        )
        $found = $false
        foreach ($pattern in $patterns) {
            if (Select-String -Path "target/doc/$docPackageName/*.html" -Pattern $pattern -Quiet 2>$null) {
                $found = $true
                break
            }
        }
        if ($found) {
            Write-Success "    Found expected: $symbol"
            $script:SuccessCount++
        } else {
            Write-Failure "    Missing expected: $symbol"
            $script:FailureCount++
            $allPassed = $false
        }
    }

    # Check unexpected symbols don't exist (look for trait/struct/type definitions or reexports)
    foreach ($symbol in $UnexpectedSymbols) {
        # Look for: trait links, struct links, type links, or reexport declarations
        $patterns = @(
            "trait\.$symbol",
            "struct\.$symbol",
            "type\.$symbol",
            "reexport\.$symbol"
        )
        $found = $false
        foreach ($pattern in $patterns) {
            if (Select-String -Path "target/doc/$docPackageName/*.html" -Pattern $pattern -Quiet 2>$null) {
                $found = $true
                break
            }
        }
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

# Runtime-gated fluxion-stream operators/types (2 operators + 2 types = 4 items)
$RuntimeGatedStreamOperators = @(
    "PartitionExt",
    "ShareExt",
    # Types
    "FluxionShared",
    "PartitionedStream"
)

# All fluxion-stream operators/types with runtime
# 20+2=22 stream operators + 3+2=5 types = 27 stream items total
$AllStreamOperators = $AllNonGatedStreamOperators + $RuntimeGatedStreamOperators

# Note: fluxion-stream-time operators (5 time operators) not tested here
# They will be tested separately after migration is complete

# fluxion-exec operators
$NonGatedExecOperators = @(
    "SubscribeExt"
)

$RuntimeGatedExecOperators = @(
    "SubscribeLatestExt"
)

$AllExecOperators = $NonGatedExecOperators + $RuntimeGatedExecOperators

# ============================================================================
# Test 1: Default features (runtime-tokio enabled)
# ============================================================================
Write-TestHeader "Test 1: Default Features (runtime-tokio)"

$result = Test-Compilation -Package "fluxion-stream" -Features "" -Description "Default features compile"
Record-Result $result

# ============================================================================
# Test 2: no_std + alloc (no runtime features)
# ============================================================================
Write-TestHeader "Test 2: no_std + alloc (no runtime features)"

$result = Test-Compilation -Package "fluxion-stream" -Features "single,alloc" -Description "no_std with alloc (single runtime)"
Record-Result $result

$result = Test-Compilation -Package "fluxion-exec" -Features "alloc" -Description "no_std exec with alloc"
Record-Result $result

# ============================================================================
# Test 3: Individual runtime features
# ============================================================================
Write-TestHeader "Test 3: Individual Runtime Features"

$result = Test-Compilation -Package "fluxion-stream" -Features "multi,runtime-tokio" -Description "runtime-tokio with multi"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "multi,runtime-smol" -Description "runtime-smol with multi"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "multi,runtime-async-std" -Description "runtime-async-std with multi"
Record-Result $result

# ============================================================================
# Test 4: Verify gated operators are excluded without runtime
# ============================================================================
Write-TestHeader "Test 4: Gated Operators Excluded (no_std + alloc)"

Write-Host "  Checking symbol presence in documentation..."
Write-Host "  Expected: $($AllNonGatedStreamOperators.Count) items present (20 operators + 3 types), $($RuntimeGatedStreamOperators.Count) items absent (2 operators + 2 types)"
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "single,alloc" `
    -ExpectedSymbols $AllNonGatedStreamOperators `
    -UnexpectedSymbols $RuntimeGatedStreamOperators `
    -Description "no_std+alloc: All non-gated operators present, runtime-gated absent"
Record-Result $result

# ============================================================================
# Test 5: Verify gated operators are included with runtime
# ============================================================================
Write-TestHeader "Test 5: Gated Operators Included (with runtime)"

Write-Host "  Checking symbol presence in documentation..."
Write-Host "  Expected: All $($AllStreamOperators.Count) items present (22 stream operators + 5 types)"
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "multi,runtime-tokio" `
    -ExpectedSymbols $AllStreamOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio: All operators including gated ones present"
Record-Result $result

# ============================================================================
# Test 6: Verify fluxion-exec gating
# ============================================================================
Write-TestHeader "Test 6: fluxion-exec Gating"

Write-Host "  Checking symbol presence in documentation..."
Write-Host "  Testing no_std+alloc: $($NonGatedExecOperators.Count) operator present, $($RuntimeGatedExecOperators.Count) operator absent"
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "alloc" `
    -ExpectedSymbols $NonGatedExecOperators `
    -UnexpectedSymbols $RuntimeGatedExecOperators `
    -Description "no_std+alloc: subscribe available, subscribe_latest absent"
Record-Result $result

Write-Host "  Testing runtime-tokio: All $($AllExecOperators.Count) operators present"
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "runtime-tokio" `
    -ExpectedSymbols $AllExecOperators `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio: All operators including subscribe_latest present"
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
