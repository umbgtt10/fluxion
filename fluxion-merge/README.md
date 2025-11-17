# fluxion-merge

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Merge multiple Fluxion streams with ordering guarantees.

## Overview

This crate provides operators for combining multiple ordered streams while preserving temporal ordering:

- **`merge`**: Combine streams emitting all items as they arrive
- **`merge_ordered`**: Merge with strict temporal ordering enforcement
- **`MergeWith`**: Fluent API for merging streams

## Key Features

- Preserves temporal ordering via sequence numbers
- Efficient concurrent stream handling
- Zero-cost abstractions over futures streams
- Integration with `FluxionStream`

## Example

```rust
use fluxion_stream::FluxionStream;
use fluxion_merge::MergeWith;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

    let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    let stream2 = FluxionStream::from_unbounded_receiver(rx2);

    let merged = stream1.merge(stream2);

    tx1.send(1)?;
    tx2.send(2)?;
    tx1.send(3)?;

    // Items arrive in temporal order based on sequence numbers
    let items: Vec<_> = merged.take(3).collect().await;

    Ok(())
}
```

## Operators

### merge

Combines multiple streams, emitting all items as they arrive while preserving ordering:

```rust
let merged = stream1.merge(stream2);
```

### merge_ordered

Enforces strict temporal ordering by buffering out-of-order items:

```rust
use fluxion_merge::merge_ordered;

let ordered = merge_ordered(vec![stream1, stream2, stream3]);
```

## Performance

The merge operators are optimized for:
- Minimal memory allocation
- Efficient concurrent polling
- Low-latency item propagation

See [benchmarks](benches/) for detailed performance characteristics.

## License

MIT
