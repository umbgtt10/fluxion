# fluxion-stream

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Stream combinators for async Rust with strong temporal-ordering guarantees. This crate provides composable operators and lightweight sequencing utilities designed for correctness and performance in event-driven systems.

[![Crates.io](https://img.shields.io/crates/v/fluxion-stream.svg)](https://crates.io/crates/fluxion-stream)
[![Documentation](https://docs.rs/fluxion-stream/badge.svg)](https://docs.rs/fluxion-stream)
[![License](https://img.shields.io/crates/l/fluxion-stream.svg)](LICENSE)

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Core Concepts](#core-concepts)
  - [Timestamped Trait](#timestamped-trait)
  - [Temporal Ordering](#temporal-ordering)
  - [Error Propagation](#error-propagation)
- [Stream Operators](#stream-operators)
  - [Combination Operators](#combination-operators)
  - [Filtering Operators](#filtering-operators)
  - [Transformation Operators](#transformation-operators)
  - [Utility Operators](#utility-operators)
- [Operator Selection Guide](#operator-selection-guide)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [Testing](#testing)
- [License](#license)

## Overview

`fluxion-stream` is a collection of reactive stream operators that maintain temporal ordering across asynchronous operations. Unlike standard stream combinators, all operators in this crate respect the intrinsic ordering of items (via timestamps, sequence numbers, or other ordering mechanisms), ensuring correct temporal sequencing even when events arrive out of order.

**Use this crate when:**
- You need to combine multiple async streams while preserving temporal order
- Events may arrive out of order and need to be resequenced
- You're building reactive systems (dashboards, monitoring, event processing)
- You need composable stream operations with correctness guarantees

## Key Features

- **Temporal Ordering**: All operators maintain temporal correctness via the `Timestamped` trait
- **Composable Operators**: 9+ stream combinators designed to work together seamlessly
- **Error Propagation**: Structured error handling through `StreamItem<T>` enum
- **Zero-Copy**: Minimal allocations and efficient buffering strategies
- **Tokio Integration**: Built on tokio streams for async runtime compatibility
- **Type Safety**: Compile-time guarantees for ordering and type compatibility

## Core Concepts

### Timestamped Trait

The `Timestamped` trait is the foundation of fluxion-stream. It provides temporal ordering for stream items:

```rust
pub trait Timestamped: Clone {
    type Inner: Clone;
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    fn timestamp(&self) -> Self::Timestamp;  // Get the timestamp for ordering
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self;
    fn with_fresh_timestamp(value: Self::Inner) -> Self;
    fn into_inner(self) -> Self::Inner;
}
```

**Implementations:**
- `Sequenced<T>` - Test utility from `fluxion-test-utils` using monotonically growing sequence numbers
- Custom domain types - Implement for your types (e.g., events with built-in timestamps)

### Temporal Ordering

Temporal ordering means items are processed based on their intrinsic timestamp, not arrival time:

```rust
// Stream 1 sends: [timestamp=2, value=B]
// Stream 2 sends: [timestamp=1, value=A]
// Merged output:  [timestamp=1, value=A], [timestamp=2, value=B]  ✓ Correct temporal order
```

**How it works:**
1. Each item has a `timestamp()` value (chrono::DateTime, u64 counter, etc.)
2. Operators buffer items and emit them in order of their `timestamp()` value
3. Late-arriving items are placed correctly in the sequence
4. Gaps in timestamps may cause buffering until the sequence is complete

**When to use:**
- Event sourcing and event-driven architectures
- Time-series data processing
- Distributed system event correlation
- Any scenario where arrival order ≠ logical order

### Error Propagation

All operators use `StreamItem<T>` for structured error handling:

```rust
pub enum StreamItem<T> {
    Value(T),      // Successful value
    Error(FluxionError),  // Error (lock failures, processing errors, etc.)
}
```

**Error handling patterns:**

```rust
// Pattern 1: Unwrap (panic on error)
let value = stream.next().await.unwrap().unwrap();

// Pattern 2: Filter errors
let values = stream
    .filter_map(|item| async move { item.ok() })
    .collect().await;

// Pattern 3: Handle explicitly
match stream.next().await {
    Some(StreamItem::Value(v)) => process(v),
    Some(StreamItem::Error(e)) => log_error(e),
    None => break,
}
```

See [Error Handling Guide](../docs/ERROR-HANDLING.md) for comprehensive patterns.

## Stream Operators

### Combination Operators

#### `combine_latest`
Combines multiple streams, emitting when any stream emits (after all have emitted once).

**Use case:** Dashboard combining data from multiple sources

```rust
use fluxion_stream::{FluxionStream, CombineLatestExt};

let combined = stream1.combine_latest(
    vec![stream2, stream3],
    |state| state.values().len() == 3  // Emit when all present
);
```

**Behavior:**
- Waits for initial values from all streams
- Emits combined state when any stream produces a value
- Maintains latest value from each stream
- Preserves temporal ordering based on triggering stream

[Full documentation](src/combine_latest.rs) | [Tests](tests/combine_latest_tests.rs)

#### `with_latest_from`
Samples secondary streams only when primary stream emits.

**Use case:** User actions enriched with latest configuration/state

```rust
use fluxion_stream::{FluxionStream, WithLatestFromExt};

let enriched = user_clicks.with_latest_from(
    vec![config_stream, state_stream],
    |combined| combined.is_complete(),
    |_primary, secondary| secondary.clone()
);
```

**Behavior:**
- Only emits when primary stream emits
- Samples latest values from secondary streams
- Primary stream drives the emission timing
- Secondary streams provide context

[Full documentation](src/with_latest_from.rs) | [Tests](tests/with_latest_from_tests.rs)

#### `ordered_merge`
Merges multiple streams preserving temporal order.

**Use case:** Event log from multiple services

```rust
use fluxion_stream::{FluxionStream, OrderedStreamExt};

let merged = stream1.ordered_merge(vec![stream2, stream3]);
```

**Behavior:**
- Emits all items from all streams
- Items emitted in order of their `timestamp()` value
- Buffers items to ensure correct ordering
- Completes when all input streams complete

[Full documentation](src/ordered_merge.rs)

### Filtering Operators

#### `emit_when`
Gates source emissions based on filter stream conditions.

**Use case:** Only emit sensor data when system is active

```rust
use fluxion_stream::{FluxionStream, EmitWhenExt};

let gated = source.emit_when(
    filter_stream,
    |filter_value| *filter_value > 0  // Predicate for gating
);
```

**Behavior:**
- Buffers source items when gate is closed
- Emits buffered items when gate opens
- Maintains temporal ordering
- Completes when source completes

[Full documentation](src/emit_when.rs) | [Tests](tests/emit_when_tests.rs)

#### `take_latest_when`
Samples source when filter condition is met.

**Use case:** Capture latest sensor reading on user request

```rust
use fluxion_stream::{FluxionStream, TakeLatestWhenExt};

let sampled = source.take_latest_when(
    trigger_stream,
    |trigger| *trigger == true
);
```

**Behavior:**
- Maintains latest value from source
- Emits latest value when filter condition is true
- Discards intermediate values (only latest matters)
- Useful for sampling / snapshot patterns

[Full documentation](src/take_latest_when.rs) | [Tests](tests/take_latest_when_tests.rs)

#### `take_while_with`
Emits while condition holds, terminates when false.

**Use case:** Process events until shutdown signal

```rust
use fluxion_stream::{FluxionStream, TakeWhileExt};

let bounded = source.take_while_with(
    condition_stream,
    |condition| *condition == true
);
```

**Behavior:**
- Emits source items while condition is true
- Terminates stream when condition becomes false
- First false terminates immediately
- Preserves temporal ordering until termination

[Full documentation](src/take_while_with.rs) | [Tests](tests/take_while_with_tests.rs)

### Transformation Operators

#### `combine_with_previous`
Pairs each value with the previous value.

**Use case:** Detect value changes or calculate deltas

```rust
use fluxion_stream::{FluxionStream, CombineWithPreviousExt};

let pairs = stream.combine_with_previous();

// Output: WithPrevious { previous: Some(1), current: 2 }
```

**Behavior:**
- First item has `previous = None`
- Subsequent items have `previous = Some(prev)`
- Useful for change detection and delta calculations
- Preserves temporal ordering

[Full documentation](src/combine_with_previous.rs) | [Tests](tests/combine_with_previous_tests.rs)

### Utility Operators

#### `map_ordered`
Maps values while preserving ordering wrapper.

```rust
let mapped = stream.map_ordered(|x| x * 2);
```

[Tests](tests/map_ordered_tests.rs)

#### `filter_ordered`
Filters values while preserving ordering wrapper.

```rust
let filtered = stream.filter_ordered(|x| *x > 10);
```

[Tests](tests/filter_ordered_tests.rs)

## Operator Selection Guide

### When You Need Combined State

| Operator | Triggers On | Output | Best For |
|----------|-------------|--------|----------|
| `combine_latest` | Any stream emits | Latest from all streams | Dashboards, state aggregation |
| `with_latest_from` | Primary emits | Primary + context | Enriching events with state |

### When You Need All Items

| Operator | Output | Ordering | Best For |
|----------|--------|----------|----------|
| `ordered_merge` | Every item | Temporal | Event logs, audit trails |
| `combine_with_previous` | Pairs (prev, curr) | Temporal | Change detection, deltas |

### When You Need Conditional Emission

| Operator | Buffering | Termination | Best For |
|----------|-----------|-------------|----------|
| `emit_when` | Yes (buffers when gated) | Source completes | Conditional processing |
| `take_latest_when` | No (only latest) | Source completes | Sampling, snapshots |
| `take_while_with` | No | First false | Bounded processing |

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
fluxion-stream = "0.2"
fluxion-core = "0.2"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

Basic usage:

```rust
use fluxion_stream::{FluxionStream, OrderedStreamExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Create channels
    let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

    // Create streams
    let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    let stream2 = FluxionStream::from_unbounded_receiver(rx2);

    // Merge in temporal order
    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Send values (out of order)
    tx2.send(Sequenced::with_sequence(100, 1)).unwrap();
    tx1.send(Sequenced::with_sequence(200, 2)).unwrap();

    // Receive in temporal order
    let first = merged.next().await.unwrap().unwrap();
    assert_eq!(first.value, 100);  // seq=1 emitted first
}
```

## Examples

### Combine Latest for Dashboard

```rust
use fluxion_stream::{FluxionStream, CombineLatestExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (cpu_tx, cpu_rx) = tokio::sync::mpsc::unbounded_channel();
    let (mem_tx, mem_rx) = tokio::sync::mpsc::unbounded_channel();

    let cpu_stream = FluxionStream::from_unbounded_receiver(cpu_rx);
    let mem_stream = FluxionStream::from_unbounded_receiver(mem_rx);

    let mut dashboard = cpu_stream.combine_latest(
        vec![mem_stream],
        |state| state.values().len() == 2
    );

    // Send metrics
    cpu_tx.send(Sequenced::with_sequence(45, 1)).unwrap();
    mem_tx.send(Sequenced::with_sequence(78, 2)).unwrap();

    // Get combined state
    if let Some(item) = dashboard.next().await {
        let state = item.unwrap();
        let values = state.get().values();
        println!("CPU: {}%, Memory: {}%", values[0], values[1]);
    }

    Ok(())
}
```

### Filter with emit_when

```rust
use fluxion_stream::{FluxionStream, EmitWhenExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (data_tx, data_rx) = tokio::sync::mpsc::unbounded_channel();
    let (gate_tx, gate_rx) = tokio::sync::mpsc::unbounded_channel();

    let data = FluxionStream::from_unbounded_receiver(data_rx);
    let gate = FluxionStream::from_unbounded_receiver(gate_rx);

    let mut gated = data.emit_when(gate, |open| *open);

    // Send data while gate is closed
    data_tx.send(Sequenced::with_sequence(1, 1)).unwrap();
    data_tx.send(Sequenced::with_sequence(2, 2)).unwrap();
    gate_tx.send(Sequenced::with_sequence(false, 3)).unwrap();

    // Open gate - buffered items released
    gate_tx.send(Sequenced::with_sequence(true, 4)).unwrap();

    // Items 1 and 2 are now emitted
    let first = gated.next().await.unwrap().unwrap();
    assert_eq!(first.value, 1);

    Ok(())
}
```

### Change Detection with combine_with_previous

```rust
use fluxion_stream::{FluxionStream, CombineWithPreviousExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let stream = FluxionStream::from_unbounded_receiver(rx);

    let mut pairs = stream.combine_with_previous();

    tx.send(Sequenced::with_sequence(10, 1)).unwrap();
    tx.send(Sequenced::with_sequence(15, 2)).unwrap();
    tx.send(Sequenced::with_sequence(15, 3)).unwrap();

    // First item - no previous
    let first = pairs.next().await.unwrap().unwrap();
    assert_eq!(first.get().current, 10);
    assert_eq!(first.get().previous, None);

    // Second item - has previous
    let second = pairs.next().await.unwrap().unwrap();
    let (prev, curr) = second.get().as_pair();
    assert_eq!(prev, Some(&10));
    assert_eq!(curr, &15);

    // Third item - detect no change
    let third = pairs.next().await.unwrap().unwrap();
    let (prev, curr) = third.get().as_pair();
    if prev == Some(curr) {
        println!("Value unchanged: {}", curr);
    }

    Ok(())
}
```

## Testing

Run all tests:

```bash
cargo test
```

Run specific operator tests:

```bash
cargo test --test combine_latest_tests
cargo test --test ordered_merge_tests
cargo test --test emit_when_tests
```

Run with error tests:

```bash
cargo test combine_latest_error_tests
```

The crate includes comprehensive test coverage for:
- Operator functionality (basic behavior)
- Error propagation scenarios
- Edge cases (empty streams, single items, etc.)
- Temporal ordering correctness
- Concurrent stream handling

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.
