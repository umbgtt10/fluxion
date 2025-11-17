# Fluxion

[![CI](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A reactive stream processing library for Rust with temporal ordering guarantees, efficient async execution and friendly fluent API.

**📊 [See why Fluxion sets new standards for quality →](PITCH.md)**

## Table of Contents
- [Features](#features)
- [Quick Start](#quick-start)
- [Operator Documentation](#-operator-documentation)
- [Core Concepts](#core-concepts)
- [Documentation](#documentation)
- [Development](#development)

## Features

- 🔄 **Rx-Style Operators**: Familiar reactive programming patterns (`combine_latest`, `with_latest_from`, `ordered_merge`, etc.)
- ⏱️ **Temporal Ordering**: Guaranteed ordering semantics with `Sequenced<T>` wrapper
- ⚡ **Async Execution**: Efficient async processing with `subscribe_async` and `subscribe_latest_async`
- 🛡️ **Type-Safe Error Handling**: Comprehensive error propagation through `StreamItem<T>` - see the [Error Handling Guide](docs/ERROR-HANDLING.md)
- 📚 **Excellent Documentation**: Detailed guides, examples, and API docs
- ✅ **Well Tested**: 1,500+ tests with comprehensive coverage


[![Crates.io](https://img.shields.io/crates/v/fluxion-rx.svg)](https://crates.io/crates/fluxion-rx)
[![Documentation](https://docs.rs/fluxion-rx/badge.svg)](https://docs.rs/fluxion-rx)

## Quick Start

Add Fluxion to your `Cargo.toml`:

```toml
[dependencies]
fluxion-rx = "0.1.0"
fluxion-test-utils = "0.1.0"
tokio = { version = "1.48", features = ["full"] }
futures = "0.3"
```

### Basic Usage

```rust
use fluxion_rx::prelude::*;
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Define enum to hold int and bool types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Bool(bool),
    }

    // Create int stream and bool trigger stream
    let (tx_int, rx_int) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();

    let int_stream = FluxionStream::from_unbounded_receiver(rx_int);
    let trigger_stream = FluxionStream::from_unbounded_receiver(rx_trigger);

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Send int values - they will be buffered
    tx_int.send(Sequenced::with_sequence(Value::Int(10), 1)).unwrap();
    tx_int.send(Sequenced::with_sequence(Value::Int(20), 2)).unwrap();
    tx_int.send(Sequenced::with_sequence(Value::Int(30), 3)).unwrap();

    // Trigger with bool - should emit latest int value (30)
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 4)).unwrap();

    let result1 = pipeline.next().await.unwrap();
    assert!(matches!(result1.get(), Value::Int(30)));
    assert_eq!(result1.sequence(), 4);

    // Send more int values - these will trigger emissions
    tx_int.send(Sequenced::with_sequence(Value::Int(40), 5)).unwrap();

    let result2 = pipeline.next().await.unwrap();
    assert!(matches!(result2.get(), Value::Int(40)));
    assert_eq!(result2.sequence(), 5);

    tx_int.send(Sequenced::with_sequence(Value::Int(50), 6)).unwrap();

    let result3 = pipeline.next().await.unwrap();
    assert!(matches!(result3.get(), Value::Int(50)));
    assert_eq!(result3.sequence(), 6);
}
```

### Chaining Multiple Operators

Fluxion operators can be chained to create complex processing pipelines. Here a complete example:

**Example: `combine_latest -> filter_ordered` - Sampling on Trigger Events**

```rust
use fluxion_rx::prelude::*;
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

#[tokio::test]
async fn test_take_latest_when_int_bool() {
    // Define enum to hold both int and string types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Str(String),
    }

    // Create two input streams
    let (tx_int, rx_int) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();
    let (tx_str, rx_str) = tokio::sync::mpsc::unbounded_channel::<Sequenced<Value>>();

    let int_stream = FluxionStream::from_unbounded_receiver(rx_int);
    let str_stream = FluxionStream::from_unbounded_receiver(rx_str);

    // Chain: combine_latest -> filter
    let mut pipeline = int_stream
        .combine_latest(vec![str_stream], |_| true)
        .filter_ordered(|combined| {
            // Keep only if first value (int) is > 50
            matches!(combined.values()[0], Value::Int(x) if x > 50)
        });

    // Send initial values
    tx_str.send(Sequenced::with_sequence(Value::Str("initial".into()), 1)).unwrap();
    tx_int.send(Sequenced::with_sequence(Value::Int(30), 2)).unwrap(); // Filtered out (30 <= 50)
    tx_int.send(Sequenced::with_sequence(Value::Int(60), 3)).unwrap(); // Passes filter (60 > 50)
    tx_str.send(Sequenced::with_sequence(Value::Str("updated".into()), 4)).unwrap(); // Passes filter (int still 60)
    tx_int.send(Sequenced::with_sequence(Value::Int(75), 5)).unwrap(); // Passes filter (75 > 50)

    // Results: seq 3 (Int 60), seq 4 (Int 60 + Str updated), seq 5 (Int 75)
    let result1 = pipeline.next().await.unwrap();
    let combined1 = result1.get().values();
    assert!(matches!(combined1[0], Value::Int(60)));
    assert!(matches!(combined1[1], Value::Str(ref s) if s == "initial"));

    let result2 = pipeline.next().await.unwrap();
    let combined2 = result2.get().values();
    assert!(matches!(combined2[0], Value::Int(60)));
    assert!(matches!(combined2[1], Value::Str(ref s) if s == "updated"));

    let result3 = pipeline.next().await.unwrap();
    let combined3 = result3.get().values();
    assert!(matches!(combined3[0], Value::Int(75)));
    assert!(matches!(combined3[1], Value::Str(ref s) if s == "updated"));
}
```

### 📚 Operator Documentation

- **[All Operators](docs/FLUXION_OPERATOR_SUMMARY.md)** - Complete operator reference
- **[Operators Roadmap](docs/FLUXION_OPERATORS_ROADMAP.md)** - Planned future operators

## Core Concepts

### Stream Operators

**Combining Streams:**
- `combine_latest` - Emit when any stream emits, with latest from all
- `with_latest_from` - Sample secondary streams on primary emission
- `ordered_merge` - Merge multiple streams preserving temporal order

**Filtering & Gating:**
- `emit_when` - Gate emissions based on a filter condition
- `take_latest_when` - Sample stream on trigger events
- `take_while_with` - Emit while condition holds

**Transformation:**
- `combine_with_previous` - Pair consecutive values
- `map_ordered` - Transform while preserving order
- `filter_ordered` - Filter while preserving order

### Async Execution

**Sequential Processing:**
```rust
use fluxion_exec::SubscribeAsyncExt;

stream
    .subscribe_async(
        |item, _token| async move {
            process(item).await?;
            Ok::<(), MyError>(())
        },
        None,
        Some(|err| eprintln!("Error: {}", err))
    )
    .await?;
```

**Latest-Value Processing (with auto-cancellation):**
```rust
use fluxion_exec::SubscribeLatestAsyncExt;

stream
    .subscribe_latest_async(
        |item, token| async move {
            expensive_operation(item, token).await?;
            Ok::<(), MyError>(())
        },
        Some(|err| eprintln!("Error: {}", err)),
        None
    )
    .await?;
```

## Documentation

### 📚 Guides

- **[Integration Guide](INTEGRATION.md)** - Learn the three patterns for integrating events (intrinsic, extrinsic, wrapper ordering)
- **[Error Handling Guide](docs/ERROR-HANDLING.md)** - Comprehensive guide to error propagation and recovery strategies
- **[fluxion-stream](fluxion-stream/README.md)** - Stream operators and composition patterns
- **[fluxion-exec](fluxion-exec/README.md)** - Async execution and subscription utilities

### 💡 Complete Example

The **[stream-aggregation](examples/stream-aggregation/)** example demonstrates production-ready patterns:

- **Real-world architecture**: 3 producers, 1 aggregator, 1 consumer
- **Ordered stream combining**: Merges sensor readings, metrics, and system events
- **Type-safe transformations**: Uses `UnboundedReceiverExt` for elegant type erasure
- **Graceful shutdown**: Proper cleanup with `CancellationToken`
- **Error handling**: Demonstrates best practices throughout

**Why this example matters:**
- Shows how all the pieces fit together in a realistic application
- Demonstrates the `into_fluxion_stream()` pattern for combining heterogeneous streams
- Illustrates proper resource management and cancellation
- Serves as a template for building your own event processing systems

Run it with: `cargo run --example stream-aggregation`

### 🔧 API Documentation

Generate and browse full API documentation:

```bash
cargo doc --no-deps --open
```

Or for specific crates:
```bash
cargo doc --package fluxion-stream --open
cargo doc --package fluxion-exec --open
```

## Development

### Prerequisites

- Rust toolchain (version pinned in `rust-toolchain.toml`)
- Cargo

### Building

```bash
# Run CI checks locally (PowerShell)
.\.ci\ci.ps1
```

### Workspace Structure

- **`fluxion-rx`** - Main crate (re-exports from other crates)
- **`fluxion-stream`** - Stream operators and combinators
- **`fluxion-exec`** - Execution utilities and subscriptions
- **`fluxion-core`** - Core utilities and traits
- **`fluxion-core`** - Core traits, types, and error handling
- **`fluxion-test-utils`** - Test helpers and fixtures
- **`fluxion-merge`** - Stream merging utilities
- **`fluxion-ordered-merge`** - Ordered merging implementation

### Development Notes

- All clippy, formatting, and documentation warnings are treated as errors in CI
- Use `.ci/coverage.ps1` to collect code coverage locally (requires `cargo-llvm-cov`)
- See [ROADMAP.md](ROADMAP.md) for planned features and release schedule

## Project Status

**Current Version:** 0.1.1

- ✅ Published to crates.io
- ✅ Core functionality complete
- ✅ Comprehensive test coverage
- ✅ Phase 1 error handling implemented
- 🚧 Phase 2 error propagation (planned for 1.0.0)
- 📝 Documentation complete for current features

See [ROADMAP.md](ROADMAP.md) for details on the path to 1.0.0.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Before submitting a PR:
1. Run tests: `cargo test --workspace`
2. Run clippy: `cargo clippy --workspace -- -D warnings`
3. Format code: `cargo fmt --all`
4. Update documentation if needed

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

Inspired by ReactiveX and other reactive programming libraries, with a focus on Rust's safety and performance characteristics.

## Security

All commits and releases are **GPG signed**.

**Key ID:** `5729DA194B0929542BF79074C2A11DED229A1E51`
**Fingerprint:** `5729 DA19 4B09 2954 2BF7 9074 C2A1 1DED 229A 1E51`
![GPG Verified](https://img.shields.io/badge/GPG-Verified-success)

## Author
Name: Umberto Gotti
Email: umberto.gotti@umbertogotti.dev
Twitter: https://x.com/GottiUmberto
