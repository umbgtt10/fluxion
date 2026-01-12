# fluxion-rx

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

The main entry point for Fluxion, providing a unified API that re-exports all stream operators and utilities.

## What is fluxion-rx?

`fluxion-rx` is the convenience crate that brings together all Fluxion components:

- **Stream operators** from [`fluxion-stream`](../fluxion-stream)
- **Core traits and types** from [`fluxion-core`](../fluxion-core)
- **Async execution** from [`fluxion-exec`](../fluxion-exec)
- **Stream merging** from [`fluxion-ordered-merge`](../fluxion-ordered-merge)

It serves as a **container crate** with no implementation code of its own - all functionality is delegated to specialized crates. This design keeps the codebase modular while providing a single, convenient import point.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxion-rx = "0.8.0"
tokio = { version = "1.48.0", features = ["full"] }
```

## Converting Channels to Streams: UnboundedReceiverExt

The primary feature unique to `fluxion-rx` is the `UnboundedReceiverExt` trait, which bridges tokio's channels with Fluxion's stream operators.

### Why UnboundedReceiverExt?

Fluxion follows a design philosophy that separates:
- **Production code**: Immutable, composable stream transformations
- **Test code**: Imperative channel operations for setup

This solves a fundamental conflict:
- Stream extensions consume `self` (ownership)
- Channel sends need `&mut self` (mutation)

### Basic Usage

```rust
use fluxion_rx::prelude::*;
use futures::channel::mpsc;
use fluxion_test_utils::Sequenced;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded::<Sequenced<i32>>();

    // Convert channel receiver to a stream
    let stream = rx.into_fluxion_stream();

    // Now you can use stream operators
    let doubled = stream.map_ordered(|x| x * 2);

    // Send some data
    tx.send(Sequenced::new(5)).unwrap();
    tx.send(Sequenced::new(10)).unwrap();
}
```

Or use `into_fluxion_stream` to transform the channel type:

```rust
use fluxion_rx::prelude::*;
use futures::channel::mpsc;
use fluxion_test_utils::Sequenced;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded::<i32>();

    // Transform raw i32 to Sequenced<i32> during stream creation
    let stream = rx.into_fluxion_stream(|x| Sequenced::new(x));

    // Now you can use stream operators
    let doubled = stream.map_ordered(|x| x * 2);

    // Send raw integers
    tx.send(5).unwrap();
    tx.send(10).unwrap();
}
```### Type Transformation with into_fluxion_stream

When combining multiple channel types, use `into_fluxion_stream` to map them to a common type:

```rust
use fluxion_rx::prelude::*;
use futures::channel::mpsc;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SensorEvent {
    Temperature(i32),
    Humidity(u32),
}

// Implement HasTimestamp for your type if it has intrinsic timestamps,
// or use Sequenced<T> wrapper for automatic timestamping

#[tokio::main]
async fn main() {
    let (tx_temp, rx_temp) = mpsc::unbounded::<i32>();
    let (tx_humid, rx_humid) = mpsc::unbounded::<u32>();

    // Map each channel to a common SensorEvent type
    let temp_stream = rx_temp.into_fluxion_stream(|t| SensorEvent::Temperature(t));
    let humid_stream = rx_humid.into_fluxion_stream(|h| SensorEvent::Humidity(h));

    // Now you can combine them
    let combined = temp_stream.combine_latest(vec![humid_stream], |_| true);
}
```

**Key Benefits:**
- **Type erasure**: Boxed streams hide implementation details
- **Heterogeneous sources**: Combine channels of different types
- **Clean separation**: Channels for setup, streams for processing

## Stream Operators

For detailed information about stream operators, see the [`fluxion-stream` README](../fluxion-stream/README.md).

Quick reference:

**Combining Streams:**
- `combine_latest` - Emit when any stream updates
- `with_latest_from` - Sample secondary streams
- `ordered_merge` - Merge preserving temporal order
- `merge_with` - Stateful event stream merging

**Filtering & Gating:**
- `filter_ordered` - Filter based on predicate
- `emit_when` - Gate emissions
- `take_latest_when` - Sample on trigger
- `take_while_with` - Emit while condition holds

**Transformation:**
- `map_ordered` - Transform values
- `combine_with_previous` - Pair consecutive values

For comprehensive operator documentation, see:
- [Operator Summary](../docs/FLUXION_OPERATOR_SUMMARY.md)
- [Operators Roadmap](../docs/FLUXION_OPERATORS_ROADMAP.md)

## Async Execution

For async processing patterns like `subscribe` and `subscribe_latest`, see the [`fluxion-exec` README](../fluxion-exec/README.md).

## Complete Example

```rust
use fluxion_rx::FluxionStream;
use futures::StreamExt;
use futures::channel::mpsc;

#[tokio::main]
async fn main() {
    // Setup channels
    let (tx_data, rx_data) = mpsc::unbounded::<i32>();
    let (tx_trigger, rx_trigger) = mpsc::unbounded::<bool>();

    // Convert to streams
    let data_stream = rx_data.into_fluxion_stream();
    let trigger_stream = rx_trigger.into_fluxion_stream();

    // Compose operators
    let mut pipeline = data_stream
        .take_latest_when(trigger_stream, |&trigger| trigger)
        .map_ordered(|x| x * 2)
        .filter_ordered(|&x| x > 10);

    // Send test data
    tx_data.send(5).unwrap();
    tx_data.send(10).unwrap();
    tx_trigger.send(true).unwrap();

    // Process stream
    if let Some(result) = pipeline.next().await {
        println!("Result: {:?}", result);
    }
}
```

## Documentation

- [Main Project README](../README.md) - Overview and getting started
- [fluxion-stream README](../fluxion-stream/README.md) - All stream operators
- [fluxion-exec README](../fluxion-exec/README.md) - Async execution patterns
- [Error Handling Guide](../docs/ERROR-HANDLING.md) - Error propagation patterns
- [Operator Summary](../docs/FLUXION_OPERATOR_SUMMARY.md) - Quick reference

## Architecture

The Fluxion project is organized into focused crates:

```
fluxion-rx          ← You are here (convenience re-exports)
├── fluxion-core           (traits, types, error handling)
├── fluxion-stream         (all stream operators)
├── fluxion-exec           (async execution utilities)
├── fluxion-ordered-merge  (ordered stream merging)
└── fluxion-test-utils     (testing utilities)
```

## License

Apache-2.0

