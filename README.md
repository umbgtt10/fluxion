# Fluxion

[![CI](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/fluxion-rx/badge.svg)](https://docs.rs/fluxion-rx)
[![codecov](https://codecov.io/gh/umbgtt10/fluxion/branch/main/graph/badge.svg)](https://codecov.io/gh/umbgtt10/fluxion)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/fluxion-rx.svg)](https://crates.io/crates/fluxion-rx)
[![Downloads](https://img.shields.io/crates/d/fluxion-rx.svg)](https://crates.io/crates/fluxion-rx)

[![Tokio](https://img.shields.io/badge/runtime-tokio-brightgreen.svg)](https://tokio.rs/)
[![smol](https://img.shields.io/badge/runtime-smol-blue.svg)](https://github.com/smol-rs/smol)
[![async-std](https://img.shields.io/badge/runtime-async--std-orange.svg)](https://async.rs/)
[![WASM](https://img.shields.io/badge/runtime-wasm-purple.svg)](https://webassembly.org/)
[![Embassy](https://img.shields.io/badge/runtime-embassy-red.svg)](https://embassy.dev/)

Fluxion-rx is a 100% Rust-idiomatic reactive streams library with temporal ordering guarantees and a fluent, ergonomic API. It emphasizes correctness and reliability, backed by 900+ comprehensive tests, a 7.6:1 test-to-code ratio, zero warnings, zero unwraps, and no unsafe code. The library includes comprehensive performance benchmarks, is published on crates.io, and demonstrates production-quality Rust engineering.

**📊 [See why Fluxion sets new standards for quality →](PITCH.md)**

## Table of Contents
- [Features](#features)
- [Independent Code Reviews](#-independent-code-reviews)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Documentation](#documentation)
- [Development](#development)

## Features

- 🔄 **Rx-Style Operators**: 27 implemented operators (42 total planned) - Familiar reactive programming patterns (`combine_latest`, `with_latest_from`, `ordered_merge`, `share`, `partition`, `tap`, etc.)
- ⏱️ **Temporal Ordering**: Guaranteed ordering semantics via `Timestamped` trait
- ⚡ **Async Execution**: Efficient async processing with `subscribe` and `subscribe_latest`
- 🌐 **Multi-Runtime Support**: Works with Tokio (default), smol, WebAssembly (WASM), Embassy (embedded), and async-std (deprecated) via Timer trait abstraction
- 🔌 **True Runtime Abstraction**: Zero-config for Tokio users, optional runtime selection supporting smol, WASM, Embassy, and async-std with automatic dead code elimination - never think about spawn/timer APIs again
- 🛡️ **Type-Safe Error Handling**: Comprehensive error propagation with `StreamItem<T>` and composable `on_error` operator - see the [Error Handling Guide](docs/ERROR-HANDLING.md)
- 📚 **Excellent Documentation**: Detailed guides, examples, and API docs
- ✅ **Well Tested**: 990+ tests with comprehensive coverage (Tokio + WASM + Smol + async-std + Embassy)

### 📋 Independent Code Reviews

- **[CHATGPT](assessments/ASSESSMENT_CHATGPT.md)**
- **[GEMINI](assessments/ASSESSMENT_GEMINI.md)**
- **[CLAUDE](assessments/ASSESSMENT_CLAUDE.md)**

### 📋 Other Assesssments

- **[Ordering](assessments/ORDERED-MERGE-VS-SELECT-ALL-PERFORMANCE-COMPARISON.md)**

### 📋 Benchmarks

[📊 View Benchmark Results](https://umbgtt10.github.io/fluxion/benchmarks/)

## Quick Start

Add Fluxion to your `Cargo.toml`:

```toml
[dependencies]
fluxion-rx = "0.6.14"
fluxion-test-utils = "0.6.14"
tokio = { version = "1.48.0", features = ["full"] }
anyhow = "1.0.100"
```

### Runtime Selection (Optional)

Fluxion defaults to **Tokio** with zero configuration. To use alternative runtimes:

```toml
# Use smol instead of tokio
fluxion-rx = { version = "0.6.14", default-features = false, features = ["runtime-smol"] }

# Use async-std (deprecated but supported)
fluxion-rx = { version = "0.6.14", default-features = false, features = ["runtime-async-std"] }

# Use Embassy for embedded/no_std + alloc (requires manual timer trait implementation)
fluxion-rx = { version = "0.6.14", default-features = false, features = ["runtime-embassy"] }

# WASM support is automatic when compiling for wasm32 target
# cargo build --target wasm32-unknown-unknown
```

**Benefits:**
- ✅ **Zero overhead**: Unused runtimes are completely excluded from compilation
- ✅ **No abstraction leakage**: Never write `tokio::spawn` or deal with timer APIs
- ✅ **Cross-platform**: Same code works on native, WASM, and embedded (Embassy)
- ✅ **Flexible choice**: Tokio, smol, async-std, WASM runtime, or Embassy for `no_std`

**Note:** Embassy requires `no_std` + `alloc` and manual timer trait implementation. See [fluxion-stream-time README](fluxion-stream-time/README.md) for details.

### Basic Usage

```rust
use async_channel::unbounded;
use fluxion_core::HasTimestamp;
use fluxion_stream::prelude::*;
use fluxion_test_utils::{unwrap_stream, Sequenced};

#[tokio::test]
async fn test_take_latest_when_int_bool() -> anyhow::Result<()> {
    // Define enum to hold int and bool types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Bool(bool),
    }

    // Create int stream and bool trigger stream
    let (tx_int, rx_int) = unbounded::<Sequenced<Value>>();
    let (tx_trigger, rx_trigger) = unbounded::<Sequenced<Value>>();

    let int_stream = rx_int.into_fluxion_stream();
    let trigger_stream = rx_trigger.into_fluxion_stream();

    let mut pipeline = int_stream.take_latest_when(trigger_stream, |_| true);

    // Send int values first - they will be buffered
    // Use realistic nanosecond timestamps
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(10), 1))?; // 1 sec
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(20), 2))?; // 2 sec
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(30), 3))?; // 3 sec

    // Trigger with bool - should emit latest int value (30) with trigger's sequence
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 4))?; // 4 sec

    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result1.value, Value::Int(30)));
    assert_eq!(result1.timestamp(), 4);

    // After first trigger, send more int values
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(40), 5))?; // 5 sec

    // Need another trigger to emit the buffered value
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 6))?; // 6 sec

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result2.value, Value::Int(40)));
    assert_eq!(result2.timestamp(), 6);
    // Send another int and trigger
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(50), 7))?; // 7 sec
    tx_trigger.try_send(Sequenced::with_timestamp(Value::Bool(true), 8))?; // 8 sec

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    assert!(matches!(&result3.value, Value::Int(50)));
    assert_eq!(result3.timestamp(), 8);
    Ok(())
}
```

### Chaining Multiple Operators

Fluxion operators can be chained to create complex processing pipelines. Here a complete example:

**Dependencies:**
```toml
[dependencies]
fluxion-rx = "0.6.14"
fluxion-test-utils = "0.6.14"
tokio = { version = "1.48.0", features = ["full"] }
anyhow = "1.0.100"
```

**Example: `combine_latest -> filter_ordered` - Sampling on Trigger Events**

```rust
use async_channel::unbounded;
use fluxion_core::Timestamped;
use fluxion_stream::prelude::*;
use fluxion_test_utils::{unwrap_stream, Sequenced};

#[tokio::test]
async fn test_combine_latest_int_string_filter_order() -> anyhow::Result<()> {
    // Define enum to hold both int and string types
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Value {
        Int(i32),
        Str(String),
    }

    // Create two input streams
    let (tx_int, rx_int) = unbounded::<Sequenced<Value>>();
    let (tx_str, rx_str) = unbounded::<Sequenced<Value>>();

    let int_stream = rx_int.into_fluxion_stream();
    let str_stream = rx_str.into_fluxion_stream();

    // Chain: combine_latest -> filter
    let mut pipeline = int_stream
        .combine_latest(vec![str_stream], |_| true)
        .filter_ordered(|combined| {
            // Keep only if first value (int) is > 50
            matches!(combined.values()[0], Value::Int(x) if x > 50)
        });

    // Send initial values
    tx_str.try_send(Sequenced::with_timestamp(Value::Str("initial".into()), 1))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(30), 2))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(60), 3))?; // Passes filter (60 > 50)
    tx_str.try_send(Sequenced::with_timestamp(Value::Str("updated".into()), 4))?;
    tx_int.try_send(Sequenced::with_timestamp(Value::Int(75), 5))?; // Passes filter (75 > 50)
                                                                    // Results: seq 3 (Int 60), seq 4 (Int 60 + Str updated), seq 5 (Int 75)
    let result1 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state1 = result1.into_inner();
    let combined1 = state1.values();
    assert!(matches!(combined1[0], Value::Int(60)));
    assert!(matches!(combined1[1], Value::Str(ref s) if s == "initial"));

    let result2 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state2 = result2.into_inner();
    let combined2 = state2.values();
    assert!(matches!(combined2[0], Value::Int(60)));
    assert!(matches!(combined2[1], Value::Str(ref s) if s == "updated"));

    let result3 = unwrap_stream(&mut pipeline, 500).await.unwrap();
    let state3 = result3.into_inner();
    let combined3 = state3.values();
    assert!(matches!(combined3[0], Value::Int(75)));
    assert!(matches!(combined3[1], Value::Str(ref s) if s == "updated"));

    Ok(())
}
```

### Stateful, Builder-like Stream Merging

The `merge_with` operator enables elegant stateful stream processing by merging multiple event streams into a single state object. Perfect for repository patterns and event sourcing:

**Dependencies:**
```toml
[dependencies]
fluxion-rx = "0.6.14"
fluxion-test-utils = "0.6.14"
tokio = { version = "1.48.0", features = ["full"] }
anyhow = "1.0.100"
```

**Example: Event Sourcing with Repository Pattern**

```rust
use fluxion_stream::MergedStream;
use fluxion_test_utils::{test_channel, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_merge_with_repository_pattern() -> anyhow::Result<()> {
    // Define domain events
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Event {
        UserCreated { id: u32, name: String },
        OrderPlaced { user_id: u32, amount: u32 },
        PaymentReceived { user_id: u32, amount: u32 },
    }

    // Repository state tracking users and orders
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
    struct Repository {
        total_users: u32,
        total_orders: u32,
        total_revenue: u32,
    }

    // Create event streams
    let (tx_users, user_stream) = test_channel::<Sequenced<Event>>();
    let (tx_orders, order_stream) = test_channel::<Sequenced<Event>>();
    let (tx_payments, payment_stream) = test_channel::<Sequenced<Event>>();

    let user_created1 = Sequenced::with_timestamp(
        Event::UserCreated {
            id: 1,
            name: "Alice".into(),
        },
        1,
    );
    let user_created2 = Sequenced::with_timestamp(
        Event::UserCreated {
            id: 2,
            name: "Bob".into(),
        },
        2,
    );
    let order_placed1 = Sequenced::with_timestamp(
        Event::OrderPlaced {
            user_id: 1,
            amount: 100,
        },
        3,
    );
    let payment_received1 = Sequenced::with_timestamp(
        Event::PaymentReceived {
            user_id: 1,
            amount: 100,
        },
        4,
    );
    let order_placed2 = Sequenced::with_timestamp(
        Event::OrderPlaced {
            user_id: 2,
            amount: 200,
        },
        5,
    );

    // Merge all event streams into a single repository state
    let mut repository_stream = MergedStream::seed::<Sequenced<Repository>>(Repository {
        total_users: 0,
        total_orders: 0,
        total_revenue: 0,
    })
    .merge_with(user_stream, |event: Event, repo: &mut Repository| {
        if let Event::UserCreated { .. } = event {
            repo.total_users += 1;
        }
        repo.clone()
    })
    .merge_with(order_stream, |event: Event, repo: &mut Repository| {
        if let Event::OrderPlaced { .. } = event {
            repo.total_orders += 1;
        }
        repo.clone()
    })
    .merge_with(payment_stream, |event: Event, repo: &mut Repository| {
        if let Event::PaymentReceived { amount, .. } = event {
            repo.total_revenue += amount;
        }
        repo.clone()
    });

    // Emit events in temporal order
    tx_users.unbounded_send(user_created1)?;
    tx_users.unbounded_send(user_created2)?;
    tx_orders.unbounded_send(order_placed1)?;
    tx_payments.unbounded_send(payment_received1)?;
    tx_orders.unbounded_send(order_placed2)?;

    // Verify repository state updates
    let state1 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state1.value.total_users, 1);
    assert_eq!(state1.value.total_orders, 0);
    assert_eq!(state1.value.total_revenue, 0);

    let state2 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state2.value.total_users, 2);

    let state3 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state3.value.total_orders, 1);

    let state4 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state4.value.total_revenue, 100);

    let state5 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state5.value.total_orders, 2);

    Ok(())
}
```

## Core Concepts

### 📚 Operator Documentation

- **[All Operators](docs/FLUXION_OPERATOR_SUMMARY.md)** - Complete operator reference
- **[Error Handling](docs/ERROR-HANDLING.md)** - Complete guide with `on_error` patterns
- **[on_error Operator](docs/FLUXION_OPERATOR_SUMMARY.md#on_error)** - Detailed specification and examples
- **[Operators Roadmap](docs/FLUXION_OPERATORS_ROADMAP.md)** - Planned future operators

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

**Error Handling:**
- `on_error` - Selectively consume or propagate errors (Chain of Responsibility)

### Async Execution

**Sequential Processing:**

**Dependencies:**
```toml
[dependencies]
fluxion-exec = "0.6.14"
tokio = { version = "1.48.0", features = ["full"] }
tokio-stream = "0.1.17"
```

**Example:**
```rust
use fluxion_core::CancellationToken;
use fluxion_exec::subscribe::SubscribeExt;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::sync::Arc;
use tokio::spawn;
use tokio_stream::StreamExt as _;

/// Example test demonstrating subscribe usage
#[tokio::test]
async fn test_subscribe_example() -> anyhow::Result<()> {
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
    let (tx, rx) = unbounded::<Item>();
    let stream = rx;

    // Step 2: Create a shared results container
    let results = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();

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
                let _ = notify_tx.unbounded_send(()); // Signal completion
                Ok::<(), TestError>(())
            }
        }
    };

    // Step 4: Subscribe to the stream
    let task = spawn(async move {
        stream
            .subscribe(process_func, |_| {}, None)
            .await
            .expect("subscribe should succeed");
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
    tx.unbounded_send(item1.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone()]);

    tx.unbounded_send(item2.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1.clone(), item2.clone()]);

    tx.unbounded_send(item3.clone())?;
    notify_rx.next().await.unwrap();
    assert_eq!(*results.lock().await, vec![item1, item2, item3]);

    // Clean up
    drop(tx);
    task.await.unwrap();

    Ok(())
}

```

**Latest-Value Processing (with auto-cancellation):**

**Dependencies:**
```toml
[dependencies]
fluxion-exec = "0.6.14"
tokio = { version = "1.48.0", features = ["full"] }
tokio-stream = "0.1.17"
```

**Example:**
```rust
use fluxion_core::CancellationToken;
use fluxion_exec::subscribe_latest::SubscribeLatestExt;
use futures::channel::mpsc::unbounded;
use futures::lock::Mutex as FutureMutex;
use std::sync::Arc;
use tokio::spawn;
use tokio_stream::StreamExt as _;

/// Example demonstrating subscribe_latest with automatic substitution
#[tokio::test]
async fn test_subscribe_latest_example() -> anyhow::Result<()> {
    #[derive(Debug, thiserror::Error)]
    #[error("Error")]
    struct Err;

    let (tx, rx) = unbounded::<u32>();
    let completed = Arc::new(FutureMutex::new(Vec::new()));
    let (notify_tx, mut notify_rx) = unbounded();
    let (gate_tx, gate_rx) = unbounded::<()>();
    let gate_rx_shared = Arc::new(FutureMutex::new(Some(gate_rx)));
    let (start_tx, mut start_rx) = unbounded::<()>();
    let start_tx_shared = Arc::new(FutureMutex::new(Some(start_tx)));

    let process = {
        let completed = completed.clone();
        move |id: u32, token: CancellationToken| {
            let completed = completed.clone();
            let notify_tx = notify_tx.clone();
            let gate_rx_shared = gate_rx_shared.clone();
            let start_tx_shared = start_tx_shared.clone();
            async move {
                if let Some(tx) = start_tx_shared.lock().await.take() {
                    tx.unbounded_send(()).ok();
                }
                if let Some(mut rx) = gate_rx_shared.lock().await.take() {
                    rx.next().await;
                }
                if !token.is_cancelled() {
                    completed.lock().await.push(id);
                    notify_tx.unbounded_send(()).ok();
                }
                Ok::<(), Err>(())
            }
        }
    };

    spawn(async move {
        rx.subscribe_latest(process, |_| {}, None).await.unwrap();
    });

    tx.unbounded_send(1)?;
    start_rx.next().await.unwrap();
    for i in 2..=5 {
        tx.unbounded_send(i)?;
    }
    gate_tx.unbounded_send(())?;
    notify_rx.next().await.unwrap();
    notify_rx.next().await.unwrap();

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
- **[fluxion-stream-time](fluxion-stream-time/README.md)** - Time-based operators (delay, debounce, throttle, sample, timeout)
- **[fluxion-exec](fluxion-exec/README.md)** - Async execution and subscription utilities
- **[fluxion-core](fluxion-core/README.md)** - Core traits, types, and utilities
- **[fluxion-ordered-merge](fluxion-ordered-merge/README.md)** - Generic ordered merging
- **[fluxion-test-utils](fluxion-test-utils/README.md)** - Testing helpers and fixtures

### 💡 Complete Examples

#### WASM Dashboard (Browser Visualization)

The **[wasm-dashboard](examples/wasm-dashboard/)** example demonstrates reactive stream operators in a browser environment:

- **Live visualization**: Real-time display of 9 streams (3 sensors + 1 combined + 5 time operators)
- **Four-layer architecture**: Source → Processing → Orchestration → UI with trait boundaries
- **Time-based operators**: Debounce, delay, sample, throttle, and timeout in action
- **Pure orchestrator**: Generic `DashboardOrchestrator` depends only on abstractions
- **WASM integration**: Rust streams compiled to WebAssembly with web-sys DOM manipulation

**Why this example matters:**
- Visual demonstration of operator behavior with real-time feedback
- Shows dependency inversion and trait-based design patterns
- Demonstrates WASM-compatible timer implementations
- Serves as educational tool for understanding reactive stream transformations

Run it with: `.\scripts\run-dashboard.ps1` (from `examples/wasm-dashboard/`)

#### Stream Aggregation (Intrinsic Timestamps)

The **[stream-aggregation](examples/stream-aggregation/)** example demonstrates production-ready patterns with intrinsic timestamps:

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

Run it with: `cargo run --bin rabbitmq_aggregator`

#### Legacy Integration (Wrapper Pattern)

The **[legacy-integration](examples/legacy-integration/)** example demonstrates integrating legacy systems using the wrapper pattern:

- **Wrapper pattern**: Add timestamps at system boundaries for sources without intrinsic ordering
- **Multiple data sources**: Database (JSON), Message Queue (XML), File Watcher (CSV)
- **Stateful aggregation**: Uses `merge_with` for repository pattern with shared state
- **Heterogeneous streams**: Unifies different event types into single aggregated stream
- **Real-time analytics**: Order tracking with live statistics and event counting
- **Graceful shutdown**: Ctrl+C handling and 1-minute auto-timeout

**Why this example matters:**
- Shows how to integrate legacy systems that lack intrinsic timestamps
- Demonstrates the adapter pattern for adding temporal ordering at boundaries
- Uses `merge_with` to build stateful repository aggregation
- Illustrates processing events with `subscribe` and cancellation tokens

Run it with: `cargo run --bin legacy-integration`

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
- **[fluxion-stream-time](fluxion-stream-time/README.md)** - Time-based operators (delay, debounce, throttle, sample, timeout)
- **[fluxion-exec](fluxion-exec/README.md)** - Execution utilities and subscriptions
- **[fluxion-core](fluxion-core/README.md)** - Core traits, types, and utilities
- **[fluxion-ordered-merge](fluxion-ordered-merge/README.md)** - Generic ordered merging implementation
- **[fluxion-test-utils](fluxion-test-utils/README.md)** - Test helpers and fixtures

See individual crate READMEs for detailed documentation.

### Development Notes

- All clippy, formatting, and documentation warnings are treated as errors in CI
- Use `.ci/coverage.ps1` to collect code coverage locally (requires `cargo-llvm-cov`)
- See [ROADMAP.md](ROADMAP.md) for planned features and release schedule

## Project Status

**Current Version:** 0.6.14

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

**Name:** Umberto Gotti

**Email:** umberto.gotti@umbertogotti.dev

**Twitter:** https://x.com/GottiUmberto

**LinkedIn:** www.linkedin.com/in/umberto-gotti-85346b48

