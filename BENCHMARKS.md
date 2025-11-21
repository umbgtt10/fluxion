# Fluxion Performance Benchmarks

> Comprehensive performance metrics for all fluxion-stream operators

---

## Overview

These benchmarks measure the throughput and latency of Fluxion's stream operators across different workload sizes and payload configurations. All benchmarks are run using [Criterion.rs](https://github.com/bheisler/criterion.rs) with:

- **100 samples** per benchmark for statistical significance
- **Payload sizes**: 0 bytes (minimal overhead) and 128 bytes (realistic data)
- **Stream sizes**: 100, 1,000, and 10,000 elements
- **Platform**: Windows x64, Release build with optimizations

## Benchmark Results

### Single-Stream Operators

These operators transform or filter a single stream.

#### `map_ordered` - Value Transformation

Transforms each value while preserving temporal order.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 940 μs | ~106K elem/s |
| 100 | 128 bytes | 968 μs | ~103K elem/s |
| 1,000 | 0 bytes | 945 μs | ~1.06M elem/s |
| 1,000 | 128 bytes | 954 μs | ~1.05M elem/s |
| 10,000 | 0 bytes | 963 μs | ~10.4M elem/s |
| 10,000 | 128 bytes | 1.41 ms | ~7.1M elem/s |

**Key Insights:**
- Excellent scaling: 10.4M elements/sec at 10K elements
- Minimal overhead for zero-byte payloads
- Payload size has negligible impact until larger volumes

#### `filter_ordered` - Conditional Filtering

Filters values based on a predicate (50% pass rate in benchmark).

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 914 μs | ~109K elem/s |
| 100 | 128 bytes | 914 μs | ~109K elem/s |
| 1,000 | 0 bytes | 960 μs | ~1.04M elem/s |
| 1,000 | 128 bytes | 961 μs | ~1.04M elem/s |
| 10,000 | 0 bytes | 1.16 ms | ~8.6M elem/s |
| 10,000 | 128 bytes | 1.89 ms | ~5.3M elem/s |

**Key Insights:**
- Comparable performance to `map_ordered`
- Predicate evaluation overhead is minimal
- Throughput counts input elements, not filtered output

---

### Multi-Stream Combination Operators

These operators combine multiple streams with various strategies.

#### `combine_latest` - Latest Values from All Streams

Emits when any stream updates, combining latest values from 3 streams.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 891 μs | ~112K elem/s |
| 100 | 128 bytes | 935 μs | ~107K elem/s |
| 1,000 | 0 bytes | 1.21 ms | ~827K elem/s |
| 1,000 | 128 bytes | 1.80 ms | ~556K elem/s |

**Key Insights:**
- State management overhead increases with payload size
- Well-optimized for real-time state synchronization
- 3 streams combined (100 elem each = 300 total coordinated)

#### `combine_with_previous` - Sliding Window Pairing

Pairs each value with its predecessor.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 912 μs | ~110K elem/s |
| 100 | 128 bytes | 992 μs | ~101K elem/s |
| 1,000 | 0 bytes | 948 μs | ~1.05M elem/s |
| 1,000 | 128 bytes | 970 μs | ~1.03M elem/s |
| 10,000 | 0 bytes | 1.13 ms | ~8.8M elem/s |
| 10,000 | 128 bytes | 1.94 ms | ~5.2M elem/s |

**Key Insights:**
- Very efficient for delta calculations and change detection
- Minimal overhead compared to single-stream operators
- Excellent scaling to high volumes

#### `with_latest_from` - Augment with Secondary Stream

Combines primary stream with latest value from secondary stream.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 872 μs | ~115K elem/s |
| 100 | 128 bytes | 867 μs | ~115K elem/s |
| 1,000 | 0 bytes | 910 μs | ~1.10M elem/s |
| 1,000 | 128 bytes | 1.06 ms | ~943K elem/s |

**Key Insights:**
- Fastest multi-stream operator for simple augmentation
- Excellent for enriching events with reference data
- Minimal latency impact

#### `merge_with` - Repository Pattern Merging

Merges 3 streams with stateful transformation (repository pattern).

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 300 total | 0 bytes | 929 μs | ~323K elem/s |
| 300 total | 128 bytes | 940 μs | ~319K elem/s |
| 3,000 total | 0 bytes | 1.08 ms | ~2.78M elem/s |
| 3,000 total | 128 bytes | 1.61 ms | ~1.86M elem/s |
| 30,000 total | 0 bytes | 3.89 ms | ~7.71M elem/s |
| 30,000 total | 128 bytes | 5.79 ms | ~5.18M elem/s |

**Key Insights:**
- Designed for complex state aggregation patterns
- Excellent throughput even with 3 merged streams
- Scales well to high volumes (30K elements in <6ms)

#### `ordered_merge` - Temporal Ordering of Multiple Streams

Merges 2-5 streams maintaining temporal order across all.

**2 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 200 total | 0 bytes | 2.32 ms | ~86K elem/s |
| 200 total | 128 bytes | 1.84 ms | ~109K elem/s |

**3 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 300 total | 0 bytes | 1.47 ms | ~204K elem/s |
| 300 total | 128 bytes | 934 μs | ~321K elem/s |

**5 Streams:**

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 500 total | 0 bytes | 2.34 ms | ~214K elem/s |
| 500 total | 128 bytes | 1.22 ms | ~410K elem/s |

**Key Insights:**
- Critical for maintaining global temporal consistency
- Performance varies with stream count and timing patterns
- Optimized priority queue implementation

---

### Conditional Emission Operators

These operators control when values are emitted based on conditions.

#### `emit_when` - Conditional Emission with Secondary Stream

Emits source values only when predicate evaluates to true based on both streams.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 932 μs | ~107K elem/s |
| 100 | 128 bytes | 927 μs | ~108K elem/s |
| 1,000 | 0 bytes | 980 μs | ~1.02M elem/s |
| 1,000 | 128 bytes | 1.24 ms | ~805K elem/s |

**Key Insights:**
- Complex predicate evaluation with minimal overhead
- Excellent for conditional event processing
- State tracking across two streams is highly optimized

#### `take_latest_when` - Sampling with Trigger Stream

Emits latest source value when trigger stream fires.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 907 μs | ~110K elem/s |
| 100 | 128 bytes | 921 μs | ~109K elem/s |
| 1,000 | 0 bytes | 931 μs | ~1.07M elem/s |
| 1,000 | 128 bytes | 1.02 ms | ~982K elem/s |
| 10,000 | 0 bytes | 1.54 ms | ~6.5M elem/s |
| 10,000 | 128 bytes | 2.89 ms | ~3.5M elem/s |

**Key Insights:**
- Efficient for sampling and debouncing patterns
- Latest-value tracking has minimal overhead
- Great for UI updates and periodic snapshots

#### `take_while_with` - Conditional Stream Termination

Continues emitting while a condition based on filter stream remains true.

| Elements | Payload | Time (avg) | Throughput |
|----------|---------|-----------|------------|
| 100 | 0 bytes | 933 μs | ~107K elem/s |
| 100 | 128 bytes | 921 μs | ~109K elem/s |
| 1,000 | 0 bytes | 930 μs | ~1.08M elem/s |
| 1,000 | 128 bytes | 996 μs | ~1.00M elem/s |
| 10,000 | 0 bytes | 1.58 ms | ~6.3M elem/s |
| 10,000 | 128 bytes | 2.45 ms | ~4.1M elem/s |

**Key Insights:**
- Minimal overhead for early termination logic
- Excellent for bounded stream processing
- Comparable performance to `take_latest_when`

---

## Performance Summary

### Throughput by Operator Category

| Category | Typical Throughput | Use Case |
|----------|-------------------|----------|
| **Single-stream transforms** | 1-10M elem/s | Basic data transformation |
| **Multi-stream combination** | 500K-8M elem/s | Event correlation, state sync |
| **Conditional emission** | 1-6M elem/s | Filtering, sampling, triggers |
| **Repository pattern** | 2-8M elem/s | Stateful aggregation |
| **Ordered merge** | 100K-400K elem/s | Temporal consistency |

### Scaling Characteristics

- **Excellent linear scaling**: Most operators maintain consistent throughput per element as volume increases
- **Payload impact**: Larger payloads (128 bytes) show 20-40% throughput reduction at high volumes
- **Multi-stream overhead**: 2-3x overhead compared to single-stream operators, but still highly performant
- **State management**: Repository pattern (`merge_with`) maintains excellent performance even with complex state

### Hardware & Environment

- **Platform**: Windows x64
- **Compiler**: Rust 1.91.0 (release mode)
- **Runtime**: Tokio async runtime
- **Measurement**: Criterion.rs with 100 samples per benchmark

---

## How to Run Benchmarks

```powershell
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
```

## Interpreting Results

- **Time (avg)**: Average execution time per benchmark iteration
- **Throughput**: Elements processed per second
- **Elements**: Number of items in the input stream(s)
- **Payload**: Size of each element's data
- **μs**: Microseconds (1/1,000,000 second)
- **ms**: Milliseconds (1/1,000 second)

Lower times and higher throughput indicate better performance.

---

*Last updated: November 21, 2025*
