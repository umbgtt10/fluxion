# fluxion-core

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Core traits and types for ordered stream processing in async Rust.

## Overview

This crate provides the foundational abstractions used throughout the Fluxion ecosystem:

- **`Timestamped` trait**: Temporal ordering for stream items via timestamps
- **`StreamItem<T>`**: Error-aware stream item wrapper
- **Lock utilities**: Safe mutex operations with error propagation

## Key Types

### Timestamped Trait

The `Timestamped` trait enables temporal ordering guarantees across stream operations:

```rust
pub trait Timestamped: Clone {
    type Inner: Clone;
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    fn timestamp(&self) -> Self::Timestamp;  // Get timestamp for ordering
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self;
    fn with_fresh_timestamp(value: Self::Inner) -> Self;
    fn into_inner(self) -> Self::Inner;
}
```

### Sequenced<T>

A ready-to-use implementation of `Timestamped` using monotonically growing sequence numbers (available in `fluxion-test-utils`):

```rust
use fluxion_test_utils::Sequenced;
use fluxion_core::Timestamped;

let item = Sequenced::new(42);
assert_eq!(item.value, 42);
// Timestamp uses chrono::Utc::now()
```

### Lock Utilities

Safe mutex lock acquisition with error propagation:

```rust
use fluxion_core::lock_utilities::lock_or_error;
use std::sync::{Arc, Mutex};

let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
let mut guard = lock_or_error(&mutex)?;
guard.push(4);
```

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxion-core = "0.2.2"
```

## License

Apache-2.0
