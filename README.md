# Fluxion

[![CI](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A reactive stream processing library for Rust with temporal ordering guarantees and efficient async execution.

## Documentation

- **[Integration Guide](INTEGRATION.md)** - Learn the three patterns for integrating events (intrinsic, extrinsic, wrapper ordering)
- **[API Documentation](https://docs.rs/fluxion)** - Detailed API reference (when published)
- **[Examples](examples/)** - Complete working examples including stream aggregation

## Features

- 🔄 **Rx-Style Operators**: Familiar reactive programming patterns (`combine_latest`, `with_latest_from`, `ordered_merge`, etc.)
- ⏱️ **Temporal Ordering**: Guaranteed ordering semantics with `Sequenced<T>` wrapper
- ⚡ **Async Execution**: Efficient async processing with `subscribe_async` and `subscribe_latest_async`
- 🛡️ **Type-Safe Error Handling**: Comprehensive error propagation with `Result` types
- 📚 **Excellent Documentation**: Detailed guides, examples, and API docs
- ✅ **Well Tested**: 1,500+ tests with comprehensive coverage

## Quick Start

Add Fluxion to your `Cargo.toml`:

```toml
[dependencies]
fluxion = "0.1.0"  # Note: Not yet published to crates.io
tokio = { version = "1.48", features = ["full"] }
futures = "0.3"
```

Basic usage:

```rust
use fluxion::FluxionStream;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel and stream
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
    let stream = FluxionStream::from_unbounded_receiver(rx);
    
    // Send some values
    tx.send(1)?;
    tx.send(2)?;
    tx.send(3)?;
    drop(tx);
    
    // Collect and process
    let values: Vec<_> = stream.collect().await;
    println!("Received: {:?}", values); // Prints: [1, 2, 3]
    
    Ok(())
}
```

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
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace --all-features

# Run CI checks locally (PowerShell)
.\.ci\ci.ps1
```

### Workspace Structure

- **`fluxion`** - Main crate (re-exports from other crates)
- **`fluxion-stream`** - Stream operators and combinators
- **`fluxion-exec`** - Execution utilities and subscriptions
- **`fluxion-core`** - Core utilities and traits
- **`fluxion-error`** - Error types and handling
- **`fluxion-test-utils`** - Test helpers and fixtures
- **`fluxion-merge`** - Stream merging utilities
- **`fluxion-ordered-merge`** - Ordered merging implementation

### Development Notes

- All clippy, formatting, and documentation warnings are treated as errors in CI
- Use `.ci/coverage.ps1` to collect code coverage locally (requires `cargo-llvm-cov`)
- See [ROADMAP.md](ROADMAP.md) for planned features and release schedule

## Project Status

**Current Version:** 0.1.0 (pre-release)

- ✅ Core functionality complete
- ✅ Comprehensive test coverage
- ✅ Phase 1 error handling implemented
- 🚧 Phase 2 error propagation in progress
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

