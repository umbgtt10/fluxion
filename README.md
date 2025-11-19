# Fluxion

[![CI](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/umbgtt10/fluxion/branch/main/graph/badge.svg)](https://codecov.io/gh/umbgtt10/fluxion)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/fluxion-rx.svg)](https://crates.io/crates/fluxion-rx)
[![Downloads](https://img.shields.io/crates/d/fluxion-rx.svg)](https://crates.io/crates/fluxion-rx)
[![Documentation](https://docs.rs/fluxion-rx/badge.svg)](https://docs.rs/fluxion-rx)

A reactive stream processing library for Rust with temporal ordering guarantee, efficient async execution and friendly fluent API.

**📊 [See why Fluxion sets new standards for quality →](PITCH.md)**

## Table of Contents
- [Features](#features)
- [Independent Code Reviews](#independent-code-reviews)
- [Quick Start](#quick-start)
- [Operator Documentation](#operator-documentation)
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

### 📋 Independent Code Reviews

- **[CHATGPT](assessments/ASSESSMENT_CHATGPT.md)**
- **[GEMINI](assessments/ASSESSMENT_GEMINI.md)**
- **[CLAUDE](assessments/ASSESSMENT_CLAUDE.md)**

### 📋 Other Assesssments

- **[Unwrap](assessments/UNWRAP_EXPECT_ASSESSMENT.md)**
- **[Mutex](assessments/RWLOCK_VS_MUTEX_ASSESSMENT.md)**

## Quick Start

Add Fluxion to your `Cargo.toml`:

```toml
[dependencies]
fluxion-rx = "0.2.1"
fluxion-test-utils = "0.2.1"
tokio = { version = "1.48.0", features = ["full"] }
anyhow = "1.0.100"
```

### Basic Usage

```rust
use fluxion_rx::FluxionStream;
use fluxion_test_utils::{sequenced::Sequenced, unwrap_stream};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::test]
async fn test_take_latest_when_int_bool() -> anyhow::Result<()> {
    // Define enum to hold int and bool types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Bool(bool),
    }

    // Create int stream and bool trigger stream
    let (tx_int, rx_int) = unbounded_channel::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = unbounded_channel::<Sequenced<Value>>();

    let int_stream = FluxionStream::from_unbounded_receiver(rx_int);
    let trigger_stream = FluxionStream::from_unbounded_receiver(rx_trigger);

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Send int values first - they will be buffered
    tx_int.send(Sequenced::with_sequence(Value::Int(10), 1))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(20), 2))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(30), 3))?;

    // Trigger with bool - should emit latest int value (30) with trigger's sequence
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 4))?;

    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result1.get(), Value::Int(30)));
    assert_eq!(result1.sequence(), 4);

    // After first trigger, send more int values
    tx_int.send(Sequenced::with_sequence(Value::Int(40), 5))?;

    // Need another trigger to emit the buffered value
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 6))?;

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result2.get(), Value::Int(40)));
    assert_eq!(result2.sequence(), 6);

    // Send another int and trigger
    tx_int.send(Sequenced::with_sequence(Value::Int(50), 7))?;
    tx_trigger.send(Sequenced::with_sequence(Value::Bool(true), 8))?;

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(result3.get(), Value::Int(50)));
    assert_eq!(result3.sequence(), 8);

    Ok(())
}
```

### Chaining Multiple Operators

Fluxion operators can be chained to create complex processing pipelines. Here a complete example:

**Dependencies:**
```toml
[dependencies]
fluxion-rx = "0.2.1"
fluxion-test-utils = "0.2.1"
tokio = { version = "1.48.0", features = ["full"] }
anyhow = "1.0.100"
```

**Example: `combine_latest -> filter_ordered` - Sampling on Trigger Events**

```rust
use fluxion_rx::{FluxionStream, Ordered};
use fluxion_test_utils::{sequenced::Sequenced, unwrap_stream};
use tokio::sync::mpsc::unbounded_channel;

#[tokio::test]
async fn test_combine_latest_int_string_filter_order() -> anyhow::Result<()> {
    // Define enum to hold both int and string types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Str(String),
    }

    // Create two input streams
    let (tx_int, rx_int) = unbounded_channel::<Sequenced<Value>>();
    let (tx_str, rx_str) = unbounded_channel::<Sequenced<Value>>();

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
    tx_str.send(Sequenced::with_sequence(Value::Str("initial".into()), 1))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(30), 2))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(60), 3))?; // Passes filter (60 > 50)
    tx_str.send(Sequenced::with_sequence(Value::Str("updated".into()), 4))?;
    tx_int.send(Sequenced::with_sequence(Value::Int(75), 5))?; // Passes filter (75 > 50)

    // Results: seq 3 (Int 60), seq 4 (Int 60 + Str updated), seq 5 (Int 75)
    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let combined1 = result1.get().values();
    assert!(matches!(combined1[0], Value::Int(60)));
    assert!(matches!(combined1[1], Value::Str(ref s) if s == "initial"));

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let combined2 = result2.get().values();
    assert!(matches!(combined2[0], Value::Int(60)));
    assert!(matches!(combined2[1], Value::Str(ref s) if s == "updated"));

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let combined3 = result3.get().values();
    assert!(matches!(combined3[0], Value::Int(75)));
    assert!(matches!(combined3[1], Value::Str(ref s) if s == "updated"));

    Ok(())
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

**Dependencies:**
```toml
[dependencies]
fluxion-exec = "0.2.1"
tokio = { version = "1.48.0", features = ["full"] }
tokio-stream = "0.1.17"
tokio-util = "0.7.17"
```

**Example:**
```rust
use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

/// Example test demonstrating subscribe_async usage
#[tokio::test]
async fn test_subscribe_async_example() -> anyhow::Result<()> {
    // Define a simple data type
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Item {
        id: u32,
        value: String,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Test error: {0}")]
    struct TestError(String);

    // Step 1: Create a stream
    let (tx, rx) = unbounded_channel::<Item>();
    let stream = UnboundedReceiverStream::new(rx);

    // Step 2: Create a shared results container
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded_channel();

    // Step 3: Define the async processing function
    let process_func = {
        let results = results.clone();
        let notify_tx = notify_tx.clone();
        move |item: Item, _ctx: CancellationToken| {
            let results = results.clone();
            let notify_tx = notify_tx.clone();
            async move {
                // Process the item (in this example, just store it)
                results.lock().await.push(item);
                let _ = notify_tx.send(()); // Signal completion
                Ok::<(), TestError>(())
            }
        }
    };

    // Step 4: Subscribe to the stream
    let task = spawn(async move {
        stream
            .subscribe_async(process_func, None, None::<fn(TestError)>)
            .await
            .expect("subscribe_async should succeed");
    });

    // Step 5: Publish items and assert results
    let item1 = Item {
        id: 1,
        value: "Alice".to_string(),
    };
    let item2 = Item {
        id: 2,
        value: "Bob".to_string(),
    };
    let item3 = Item {
        id: 3,
        value: "Charlie".to_string(),
    };

    // Act & Assert
    tx.send(item1.clone())?;
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone()]);

    tx.send(item2.clone())?;
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone(), item2.clone()]);

    tx.send(item3.clone())?;
    notify_rx.recv().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1, item2, item3]);

    // Clean up
    drop(tx);
    task.await.unwrap();

    Ok(())
}

```

**Latest-Value Processing (with auto-cancellation):**

See [subscribe_latest_async_example.rs](fluxion-exec/tests/subscribe_latest_async_example.rs) for the source.

**Dependencies:**
```toml
[dependencies]
fluxion-exec = "0.2.1"
tokio = { version = "1.48.0", features = ["full"] }
tokio-stream = "0.1.17"
tokio-util = "0.7.17"
```

**Example:**
```rust
use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

/// Example demonstrating subscribe_latest_async with automatic substitution
#[tokio::test]
async fn test_subscribe_latest_async_example() -> anyhow::Result<()> {
    #[derive(Debug, thiserror::Error)]
    #[error("Error")]
    struct Err;

    let (tx, rx) = unbounded_channel::<u32>();
    let completed = Arc::new(TokioMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded_channel();
    let (gate_tx, gate_rx) = unbounded_channel::<()>();
    let gate_rx_shared = Arc::new(TokioMutex::new(Some(gate_rx)));
    let (start_tx, mut start_rx) = unbounded_channel::<()>();
    let start_tx_shared = Arc::new(TokioMutex::new(Some(start_tx)));

    let process = {
        let completed = completed.clone();
        move |id: u32, token: CancellationToken| {
            let completed = completed.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    tx.send(()).ok();
                }
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    rx.recv().await;
                }
                if !token.is_cancelled() {
                    completed.lock().await.push(id);
                    notify_tx.send(()).ok();
                }
                Ok::<(), Err>(())
            }
        }
    };

    spawn(async move {
        UnboundedReceiverStream::new(rx)
            .subscribe_latest_async(process, None::<fn(Err)>, None)
            .await
            .unwrap();
    });

    tx.send(1)?;
    start_rx.recv().await.unwrap();
    for i in 2..=5 {
        tx.send(i)?;
    }
    gate_tx.send(())?;
    notify_rx.recv().await.unwrap();
    notify_rx.recv().await.unwrap();

    let result = completed.lock().await;
    assert_eq!(*result, vec![1, 5]);

    Ok(())
}

```

## Documentation

### 📚 Guides

- **[Integration Guide](INTEGRATION.md)** - Learn the three patterns for integrating events (intrinsic, extrinsic, wrapper ordering)
- **[Error Handling Guide](docs/ERROR-HANDLING.md)** - Comprehensive guide to error propagation and recovery strategies

### 📦 Crate Documentation

- **[fluxion-rx](fluxion/README.md)** - Main convenience crate (re-exports all operators)
- **[fluxion-stream](fluxion-stream/README.md)** - Stream operators and composition patterns
- **[fluxion-exec](fluxion-exec/README.md)** - Async execution and subscription utilities
- **[fluxion-core](fluxion-core/README.md)** - Core traits, types, and utilities
- **[fluxion-merge](fluxion-merge/README.md)** - Stream merging operators
- **[fluxion-ordered-merge](fluxion-ordered-merge/README.md)** - Generic ordered merging
- **[fluxion-test-utils](fluxion-test-utils/README.md)** - Testing helpers and fixtures

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

Run it with: `cargo run --package rabbitmq-aggregator-example`

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

This repository is organized as a Cargo workspace with the following crates:

- **[fluxion-rx](fluxion/README.md)** - Main convenience crate (re-exports from other crates)
- **[fluxion-stream](fluxion-stream/README.md)** - Stream operators and combinators
- **[fluxion-exec](fluxion-exec/README.md)** - Execution utilities and subscriptions
- **[fluxion-core](fluxion-core/README.md)** - Core traits, types, and utilities
- **[fluxion-merge](fluxion-merge/README.md)** - Stream merging operators
- **[fluxion-ordered-merge](fluxion-ordered-merge/README.md)** - Generic ordered merging implementation
- **[fluxion-test-utils](fluxion-test-utils/README.md)** - Test helpers and fixtures

See individual crate READMEs for detailed documentation.

### Development Notes

- All clippy, formatting, and documentation warnings are treated as errors in CI
- Use `.ci/coverage.ps1` to collect code coverage locally (requires `cargo-llvm-cov`)
- See [ROADMAP.md](ROADMAP.md) for planned features and release schedule

## Project Status

**Current Version:** 0.2.1

- ✅ Published to crates.io
- ✅ Core functionality complete
- ✅ Comprehensive test coverage
- ✅ Error propagation and handling implemented
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
LinkedIn: www.linkedin.com/in/umberto-gotti-85346b48
