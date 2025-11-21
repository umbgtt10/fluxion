# fluxion-ordered-merge

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Generic ordered stream merging utilities for async Rust.

## Overview

This crate provides low-level utilities for merging async streams with temporal ordering guarantees. Unlike `fluxion-merge`, which is specialized for Fluxion streams, this crate works with any stream type implementing the `Timestamped` trait.

## Features

- Generic over any `Timestamped` type
- Strict temporal ordering via buffering
- Efficient out-of-order handling
- Zero-copy stream merging where possible

## Usage

This crate is primarily used as a building block for higher-level merge operators. Most users should use `fluxion-merge` instead.

### Example

```rust
use fluxion_ordered_merge::ordered_merge;
use fluxion_test_utils::Sequenced;
use fluxion_core::Timestamped;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Create timestamped streams
    let stream1 = futures::stream::iter(vec![
        Sequenced::with_timestamp(1, 1),
        Sequenced::with_timestamp(3, 3),
    ]);

    let stream2 = futures::stream::iter(vec![
        Sequenced::with_timestamp(2, 2),
        Sequenced::with_timestamp(4, 4),
    ]);

    // Merge with ordering guarantees
    let merged = ordered_merge(vec![stream1, stream2]);

    // Items emitted in sequence order: 1, 2, 3, 4
    let items: Vec<_> = merged.collect().await;
}
```

## How It Works

The ordered merge algorithm:

1. Polls all input streams concurrently
2. Buffers items that arrive out of order
3. Emits items strictly by sequence number
4. Handles stream completion correctly

This ensures temporal ordering even when upstream streams emit at different rates or out of sequence.

## Performance

- **Memory**: Buffers only out-of-order items
- **Latency**: Minimal overhead for in-order streams
- **Throughput**: Optimized polling and buffering

## License

Apache-2.0
