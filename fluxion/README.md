# fluxion-rx

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

The main convenience crate for Fluxion, re-exporting all stream operators and utilities with a friendly interface.

## Overview

This crate provides a unified API for working with reactive streams in Rust. It brings together:

- Stream combinators from `fluxion-stream`
- Core traits and types from `fluxion-core`
- Execution utilities from `fluxion-exec`
- Merge operators from `fluxion-merge` and `fluxion-ordered-merge`

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxion-rx = "0.8"
```

## Example

```rust
use fluxion_rx::FluxionStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

    let stream1 = FluxionStream::from_unbounded_receiver(rx1);
    let stream2 = FluxionStream::from_unbounded_receiver(rx2);

    let combined = stream1.combine_latest(stream2);

    tx1.send(1).unwrap();
    tx2.send("hello").unwrap();

    // Process combined stream...
}
```

## Features

- **Temporal ordering**: Strong guarantees via sequence numbers
- **Composable operators**: `combine_latest`, `with_latest_from`, `merge`, and more
- **Efficient**: Minimal allocations, optimized for performance
- **Well-tested**: Extensive test coverage including edge cases

## Documentation

For detailed documentation, see:
- [Main README](../README.md)
- [Operator Summary](../docs/FLUXION_OPERATOR_SUMMARY.md)
- [Error Handling Guide](../docs/ERROR-HANDLING.md)

## License

MIT
