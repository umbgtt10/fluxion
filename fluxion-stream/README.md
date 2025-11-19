# fluxion-stream

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Stream combinators for async Rust with strong temporal-ordering guarantees. This crate provides composable operators and lightweight sequencing utilities designed for correctness and performance in event-driven systems.

## Key features

- Temporal ordering via `Ordered` trait and sequence numbers
- Composable operators: `combine_latest`, `with_latest_from`, `ordered_merge`, `take_latest_when`, `take_while_with`, and more
- Efficient implementation with minimal allocations
- Integration with tokio streams

## Core concepts

### Ordered Trait

The `Ordered` trait is central to fluxion-stream. It provides a temporal ordering mechanism for stream items:

```rust
pub trait Ordered: Clone {
    type Inner: Clone;

    fn order(&self) -> u64;  // Temporal ordering value
    fn get(&self) -> &Self::Inner;  // Access inner value
    fn with_order(value: Self::Inner, order: u64) -> Self;
}
```

### Stream Operators

All operators preserve temporal ordering and handle concurrent streams correctly.

## Quick Examples

### Basic Stream Creation

```rust
use fluxion_stream::FluxionStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
    let stream = FluxionStream::from_unbounded_receiver(rx);

    tx.send(1).unwrap();
    tx.send(2).unwrap();
    drop(tx);

    let values: Vec<_> = stream.collect().await;
    println!("{:?}", values); // [1, 2]
}
```

### Combining Streams with `ordered_merge`

```rust
use fluxion_stream::{FluxionStream, OrderedStreamExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Create test streams with ordered values
    let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

    let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    let stream2 = FluxionStream::from_unbounded_receiver(rx2);

    // Merge streams in temporal order
    let merged = stream1.ordered_merge(vec![stream2]);
    let values: Vec<_> = merged.map(|s| s.value).collect().await;

    // Send ordered values
    tx1.send(Sequenced::with_sequence(1, 1)).unwrap();  // (value, sequence)
    tx2.send(Sequenced::with_sequence(2, 2)).unwrap();
    tx1.send(Sequenced::with_sequence(3, 3)).unwrap();

    drop(tx1);
    drop(tx2);

    println!("{:?}", values); // [1, 2, 3] - ordered by sequence number
}
```

### Using `combine_latest`

```rust
use fluxion_stream::{FluxionStream, CombineLatestExt};
use futures::StreamExt;

// Combine multiple streams, emitting when any stream emits
// (after all have emitted at least once)
let combined = stream1.combine_latest(
    vec![stream2, stream3],
    |state| state.values().len() == 3  // Filter: all streams present
);
```

## Core modules

- `fluxion_stream` � Main `FluxionStream` type with extension methods
- Operator modules: `combine_latest`, `ordered_merge`, `with_latest_from`, `take_latest_when`,
  `take_while_with`, `emit_when`, `combine_with_previous`

## Running tests

License

Apache-2.0
