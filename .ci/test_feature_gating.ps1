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

    # Check expected symbols exist (look for re-export declarations)
    foreach ($symbol in $ExpectedSymbols) {
        $found = Select-String -Path "target/doc/$docPackageName/*.html" -Pattern "id=.reexport\.$symbol" -Quiet 2>$null
        if ($found) {
            Write-Success "    Found expected: $symbol"
        } else {
            Write-Failure "    Missing expected: $symbol"
            $allPassed = $false
        }
    }

    # Check unexpected symbols don't exist (look for re-export declarations)
    foreach ($symbol in $UnexpectedSymbols) {
        $found = Select-String -Path "target/doc/$docPackageName/*.html" -Pattern "id=.reexport\.$symbol" -Quiet 2>$null
        if (-not $found) {
            Write-Success "    Correctly excluded: $symbol"
        } else {
            Write-Failure "    Should not have: $symbol"
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
# Test 1: Default features (runtime-tokio enabled)
# ============================================================================
Write-TestHeader "Test 1: Default Features (runtime-tokio)"

$result = Test-Compilation -Package "fluxion-stream" -Features "" -Description "Default features compile"
Record-Result $result

# ============================================================================
# Test 2: no_std + alloc (no runtime features)
# ============================================================================
Write-TestHeader "Test 2: no_std + alloc (no runtime features)"

$result = Test-Compilation -Package "fluxion-stream" -Features "alloc" -Description "no_std with alloc"
Record-Result $result

$result = Test-Compilation -Package "fluxion-exec" -Features "alloc" -Description "no_std exec with alloc"
Record-Result $result

# ============================================================================
# Test 3: Individual runtime features
# ============================================================================
Write-TestHeader "Test 3: Individual Runtime Features"

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-tokio" -Description "runtime-tokio only"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-smol" -Description "runtime-smol only"
Record-Result $result

$result = Test-Compilation -Package "fluxion-stream" -Features "runtime-async-std" -Description "runtime-async-std only"
Record-Result $result

# ============================================================================
# Test 4: Verify gated operators are excluded without runtime
# ============================================================================
Write-TestHeader "Test 4: Gated Operators Excluded (no runtime)"

Write-Host "  Checking symbol presence in documentation..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "alloc" `
    -ExpectedSymbols @("FilterOrderedExt", "MapOrderedExt", "ScanOrderedExt") `
    -UnexpectedSymbols @("ShareExt", "PartitionExt") `
    -Description "no_std+alloc (no share/partition)"
Record-Result $result

# ============================================================================
# Test 5: Verify gated operators are included with runtime
# ============================================================================
Write-TestHeader "Test 5: Gated Operators Included (with runtime)"

Write-Host "  Checking symbol presence in documentation..."
$result = Test-SymbolPresence `
    -Package "fluxion-stream" `
    -Features "runtime-tokio" `
    -ExpectedSymbols @("FilterOrderedExt", "MapOrderedExt", "ShareExt", "PartitionExt") `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio (includes share/partition)"
Record-Result $result

# ============================================================================
# Test 6: Verify fluxion-exec gating
# ============================================================================
Write-TestHeader "Test 6: fluxion-exec Gating"

Write-Host "  Checking symbol presence in documentation..."
$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "alloc" `
    -ExpectedSymbols @("SubscribeExt") `
    -UnexpectedSymbols @("SubscribeLatestExt") `
    -Description "no_std+alloc (no subscribe_latest)"
Record-Result $result

$result = Test-SymbolPresence `
    -Package "fluxion-exec" `
    -Features "runtime-tokio" `
    -ExpectedSymbols @("SubscribeExt", "SubscribeLatestExt") `
    -UnexpectedSymbols @() `
    -Description "runtime-tokio (includes subscribe_latest)"
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
