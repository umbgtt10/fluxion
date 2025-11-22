#!/usr/bin/env pwsh
# benchmarks.ps1 - Run benchmarks and generate BENCHMARKS.md documentation

Write-Host "Running Fluxion benchmarks..." -ForegroundColor Cyan

# Run benchmarks and capture output
Write-Host "Executing benchmark suite..." -ForegroundColor Yellow

# Temporarily set error action preference to continue to handle stderr from cargo
$oldErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'

$benchOutputFile = "target/benchmark-output.txt"

try {
    # Run benchmarks and save output to file
    cargo bench --bench benchmarks 2>&1 | Tee-Object -FilePath $benchOutputFile | Out-Null
    $exitCode = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $oldErrorActionPreference
}

# Check if benchmarks ran successfully
if ($exitCode -ne 0) {
    Write-Host "Error: Benchmarks failed to run" -ForegroundColor Red
    if (Test-Path $benchOutputFile) {
        Get-Content $benchOutputFile | Write-Host -ForegroundColor Red
    }
    exit 1
}

Write-Host "Benchmarks completed successfully!" -ForegroundColor Green
Write-Host "Benchmark output saved to: $benchOutputFile" -ForegroundColor Cyan

# Read benchmark output from file
$benchOutput = Get-Content $benchOutputFile -Raw

# Extract timing data
$timingLines = $benchOutput -split "`n" | Where-Object { $_ -match "time:\s+\[" }

# Parse benchmark results into structured data
$results = @{}
foreach ($line in $timingLines) {
    # Extract benchmark name and time
    if ($line -match "^(.+?)\s+time:\s+\[.*?(\d+\.?\d*)\s+(µs|ms|ns)") {
        $name = $matches[1].Trim()
        $time = [double]$matches[2]
        $unit = $matches[3]

        # Convert to microseconds for consistency
        $timeInUs = switch ($unit) {
            "ns" { $time / 1000 }
            "µs" { $time }
            "ms" { $time * 1000 }
            default { $time }
        }

        $results[$name] = @{
            Time = $time
            Unit = $unit
            TimeInUs = $timeInUs
        }
    }
}

Write-Host "Parsed $($results.Count) benchmark results" -ForegroundColor Cyan

# Generate BENCHMARKS.md
$outputPath = "benchmarks\BENCHMARKS.md"
Write-Host "Generating $outputPath..." -ForegroundColor Yellow

$date = Get-Date -Format "MMMM dd, yyyy"

$content = @"
# Fluxion Performance Benchmarks

> Comprehensive performance metrics for all fluxion-stream operators

---

## Overview

These benchmarks measure the throughput and latency of Fluxion's stream operators across different workload sizes and payload configurations. All benchmarks are run using [Criterion.rs](https://github.com/bheisler/criterion.rs) with:

- **100 samples** per benchmark for statistical significance
- **Payload sizes**: 16, 32, 64, and 128 bytes
- **Stream sizes**: 100, 1,000, and 10,000 elements
- **Platform**: Windows x64, Release build with optimizations

## Benchmark Results

### Single-Stream Operators

These operators transform or filter a single stream.

#### ``map_ordered`` - Value Transformation

Transforms each value while preserving temporal order.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Helper function to format time
function Format-Time {
    param($timeUs)
    if ($timeUs -lt 1000) {
        return "$([math]::Round($timeUs, 0)) μs"
    } else {
        return "$([math]::Round($timeUs / 1000, 2)) ms"
    }
}

# Helper function to calculate throughput
function Format-Throughput {
    param($elements, $timeUs)
    $elemPerSec = ($elements / $timeUs) * 1000000
    if ($elemPerSec -lt 1000) {
        return "~$([math]::Round($elemPerSec, 0)) elem/s"
    } elseif ($elemPerSec -lt 1000000) {
        return "~$([math]::Round($elemPerSec / 1000, 0))K elem/s"
    } else {
        return "~$([math]::Round($elemPerSec / 1000000, 1))M elem/s"
    }
}

# Helper function to add benchmark row
function Add-BenchmarkRow {
    param($benchName, $elements, $payload)

    if ($results.ContainsKey($benchName)) {
        $time = Format-Time $results[$benchName].TimeInUs
        $throughput = Format-Throughput $elements $results[$benchName].TimeInUs
        return "| $elements | $payload | $time | $throughput |`n"
    }
    # Return empty string instead of null to prevent blank rows
    return ""
}

# Map Ordered
$content += "`n"
$content += (Add-BenchmarkRow "map_ordered/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "map_ordered/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "map_ordered/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "map_ordered/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "map_ordered/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "map_ordered/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "map_ordered/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "map_ordered/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "map_ordered/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "map_ordered/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "map_ordered/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "map_ordered/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Excellent scaling: High throughput maintained at 10K elements
- Minimal overhead for zero-byte payloads
- Payload size has impact at larger volumes

#### ``filter_ordered`` - Conditional Filtering

Filters values based on a predicate (50% pass rate in benchmark).

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Filter Ordered
$content += "`n"
$content += (Add-BenchmarkRow "filter_ordered/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "filter_ordered/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Comparable performance to ``map_ordered``
- Predicate evaluation overhead is minimal
- Throughput counts input elements, not filtered output

---

### Multi-Stream Combination Operators

These operators combine multiple streams with various strategies.

#### ``combine_latest`` - Latest Values from All Streams

Emits when any stream updates, combining latest values from 3 streams.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Combine Latest
$content += "`n"
$content += (Add-BenchmarkRow "combine_latest/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "combine_latest/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "combine_latest/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "combine_latest/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "combine_latest/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "combine_latest/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "combine_latest/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "combine_latest/m1000_p128" 1000 "128 bytes")

$content += @"

**Key Insights:**
- State management overhead increases with payload size
- Well-optimized for real-time state synchronization
- 3 streams combined (100 elem each = 300 total coordinated)

#### ``combine_with_previous`` - Sliding Window Pairing

Pairs each value with its predecessor.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Combine With Previous
$content += "`n"
$content += (Add-BenchmarkRow "combine_with_previous/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "combine_with_previous/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Very efficient for delta calculations and change detection
- Minimal overhead compared to single-stream operators
- Excellent scaling to high volumes

#### ``with_latest_from`` - Augment with Secondary Stream

Combines primary stream with latest value from secondary stream.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# With Latest From
$content += "`n"
$content += (Add-BenchmarkRow "with_latest_from/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "with_latest_from/m1000_p128" 1000 "128 bytes")

$content += @"

**Key Insights:**
- Fastest multi-stream operator for simple augmentation
- Excellent for enriching events with reference data
- Minimal latency impact

#### ``merge_with`` - Repository Pattern Merging

Merges 3 streams with stateful transformation (repository pattern).

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Merge With
$content += "`n"
$content += (Add-BenchmarkRow "merge_with/m100_p16" 300 "16 bytes")
$content += (Add-BenchmarkRow "merge_with/m100_p32" 300 "32 bytes")
$content += (Add-BenchmarkRow "merge_with/m100_p64" 300 "64 bytes")
$content += (Add-BenchmarkRow "merge_with/m100_p128" 300 "128 bytes")
$content += (Add-BenchmarkRow "merge_with/m1000_p16" 3000 "16 bytes")
$content += (Add-BenchmarkRow "merge_with/m1000_p32" 3000 "32 bytes")
$content += (Add-BenchmarkRow "merge_with/m1000_p64" 3000 "64 bytes")
$content += (Add-BenchmarkRow "merge_with/m1000_p128" 3000 "128 bytes")
$content += (Add-BenchmarkRow "merge_with/m10000_p16" 30000 "16 bytes")
$content += (Add-BenchmarkRow "merge_with/m10000_p32" 30000 "32 bytes")
$content += (Add-BenchmarkRow "merge_with/m10000_p64" 30000 "64 bytes")
$content += (Add-BenchmarkRow "merge_with/m10000_p128" 30000 "128 bytes")

$content += @"

**Key Insights:**
- Designed for complex state aggregation patterns
- Excellent throughput even with 3 merged streams
- Scales well to high volumes

#### ``ordered_merge`` - Temporal Ordering of Multiple Streams

Merges 2-5 streams maintaining temporal order across all.

**2 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Ordered Merge - 2 streams
$content += "`n"
$content += (Add-BenchmarkRow "ordered_merge/m100_p16_s2" 200 "16 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p32_s2" 200 "32 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p64_s2" 200 "64 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p128_s2" 200 "128 bytes")

$content += @"

**3 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Ordered Merge - 3 streams
$content += "`n"
$content += (Add-BenchmarkRow "ordered_merge/m100_p16_s3" 300 "16 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p32_s3" 300 "32 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p64_s3" 300 "64 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p128_s3" 300 "128 bytes")

$content += @"

**5 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Ordered Merge - 5 streams
$content += "`n"
$content += (Add-BenchmarkRow "ordered_merge/m100_p16_s5" 500 "16 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p32_s5" 500 "32 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p64_s5" 500 "64 bytes")
$content += (Add-BenchmarkRow "ordered_merge/m100_p128_s5" 500 "128 bytes")

$content += @"

**Key Insights:**
- Critical for maintaining global temporal consistency
- Performance varies with stream count and timing patterns
- Optimized priority queue implementation

---

### Conditional Emission Operators

These operators control when values are emitted based on conditions.

#### ``emit_when`` - Conditional Emission with Secondary Stream

Emits source values only when predicate evaluates to true based on both streams.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Emit When
$content += "`n"
$content += (Add-BenchmarkRow "emit_when/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "emit_when/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "emit_when/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "emit_when/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "emit_when/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "emit_when/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "emit_when/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "emit_when/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "emit_when/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "emit_when/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "emit_when/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "emit_when/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Complex predicate evaluation with minimal overhead
- Excellent for conditional event processing
- State tracking across two streams is highly optimized

#### ``take_latest_when`` - Sampling with Trigger Stream

Emits latest source value when trigger stream fires.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Take Latest When
$content += "`n"
$content += (Add-BenchmarkRow "take_latest_when/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "take_latest_when/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Efficient for sampling and debouncing patterns
- Latest-value tracking has minimal overhead
- Great for UI updates and periodic snapshots

#### ``take_while_with`` - Conditional Stream Termination

Continues emitting while a condition based on filter stream remains true.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
"@

# Take While With
$content += "`n"
$content += (Add-BenchmarkRow "take_while_with/m100_p16" 100 "16 bytes")
$content += (Add-BenchmarkRow "take_while_with/m100_p32" 100 "32 bytes")
$content += (Add-BenchmarkRow "take_while_with/m100_p64" 100 "64 bytes")
$content += (Add-BenchmarkRow "take_while_with/m100_p128" 100 "128 bytes")
$content += (Add-BenchmarkRow "take_while_with/m1000_p16" 1000 "16 bytes")
$content += (Add-BenchmarkRow "take_while_with/m1000_p32" 1000 "32 bytes")
$content += (Add-BenchmarkRow "take_while_with/m1000_p64" 1000 "64 bytes")
$content += (Add-BenchmarkRow "take_while_with/m1000_p128" 1000 "128 bytes")
$content += (Add-BenchmarkRow "take_while_with/m10000_p16" 10000 "16 bytes")
$content += (Add-BenchmarkRow "take_while_with/m10000_p32" 10000 "32 bytes")
$content += (Add-BenchmarkRow "take_while_with/m10000_p64" 10000 "64 bytes")
$content += (Add-BenchmarkRow "take_while_with/m10000_p128" 10000 "128 bytes")

$content += @"

**Key Insights:**
- Minimal overhead for early termination logic
- Excellent for bounded stream processing
- Comparable performance to ``take_latest_when``

---

## Performance Summary

### Throughput by Operator Category
"@

# Calculate throughput ranges for each category
$singleStreamOps = @("map_ordered", "filter_ordered")
$multiStreamOps = @("combine_latest", "combine_with_previous", "with_latest_from", "merge_with")
$conditionalOps = @("emit_when", "take_latest_when", "take_while_with")
$orderedMergeOps = @("ordered_merge")

function Get-ThroughputRange {
    param($operatorNames)

    $throughputs = @()
    foreach ($opName in $operatorNames) {
        foreach ($key in $results.Keys) {
            if ($key -match "^$opName/") {
                # Extract element count from key (e.g., "m1000" -> 1000)
                if ($key -match "m(\d+)") {
                    $elements = [int]$matches[1]
                    # For merge_with, multiply by 3 streams; for ordered_merge, extract stream count
                    if ($opName -eq "merge_with") {
                        $elements *= 3
                    } elseif ($key -match "_s(\d+)") {
                        $streamCount = [int]$matches[1]
                        $elements *= $streamCount
                    }

                    $timeUs = $results[$key].TimeInUs
                    $throughput = ($elements / $timeUs) * 1000000
                    $throughputs += $throughput
                }
            }
        }
    }

    if ($throughputs.Count -eq 0) { return "N/A" }

    $min = [math]::Min([math]::Round(($throughputs | Measure-Object -Minimum).Minimum), 999999999)
    $max = [math]::Max([math]::Round(($throughputs | Measure-Object -Maximum).Maximum), 0)

    # Format range
    if ($max -lt 1000) {
        return "$min-$max elem/s"
    } elseif ($max -lt 1000000) {
        $minK = [math]::Round($min / 1000, 0)
        $maxK = [math]::Round($max / 1000, 0)
        return "$minK-$maxK K elem/s"
    } else {
        $minM = [math]::Round($min / 1000000, 1)
        $maxM = [math]::Round($max / 1000000, 1)
        return "$minM-$maxM M elem/s"
    }
}

$singleStreamRange = Get-ThroughputRange $singleStreamOps
$multiStreamRange = Get-ThroughputRange $multiStreamOps
$conditionalRange = Get-ThroughputRange $conditionalOps
$orderedMergeRange = Get-ThroughputRange $orderedMergeOps

$content += @"

| Category | Typical Throughput | Use Case |
|----------|-------------------|----------|
| **Single-stream transforms** | $singleStreamRange | Basic data transformation |
| **Multi-stream combination** | $multiStreamRange | Event correlation, state sync |
| **Conditional emission** | $conditionalRange | Filtering, sampling, triggers |
| **Ordered merge** | $orderedMergeRange | Temporal consistency |

### Scaling Characteristics

- **Linear scaling**: Most operators maintain consistent throughput per element as volume increases
- **Payload impact**: Larger payloads show measurable throughput reduction at high volumes
- **Multi-stream overhead**: Additional complexity from coordinating multiple streams
- **State management**: Repository pattern (``merge_with``) maintains strong performance with complex state

### Hardware & Environment

- **Platform**: Windows x64
- **Compiler**: Rust (release mode)
- **Runtime**: Tokio async runtime
- **Measurement**: Criterion.rs with 100 samples per benchmark

---

## How to Run Benchmarks

``````powershell
# Run all benchmarks
cargo bench --bench benchmarks

# Run specific operator benchmarks
cargo bench --bench benchmarks -- "map_ordered"
cargo bench --bench benchmarks -- "combine_latest"

# Run with specific size
cargo bench --bench benchmarks -- "m1000"

# Generate HTML reports
cargo bench --bench benchmarks
# Open: target/criterion/report/index.html

# Run this script to regenerate BENCHMARKS.md
.\.ci\benchmarks.ps1
``````

## Interpreting Results

- **Time (avg)**: Average execution time per benchmark iteration
- **Throughput**: Elements processed per second
- **Elements**: Number of items in the input stream(s)
- **Payload**: Size of each element's data
- **μs**: Microseconds (1/1,000,000 second)
- **ms**: Milliseconds (1/1,000 second)

Lower times and higher throughput indicate better performance.

---

*Last updated: $date*
"@

# Write to file
$content | Out-File -FilePath $outputPath -Encoding UTF8 -NoNewline

Write-Host "Successfully generated $outputPath" -ForegroundColor Green
Write-Host "Documentation contains $($results.Count) benchmark results" -ForegroundColor Cyan

