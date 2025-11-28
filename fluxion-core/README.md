# fluxion-core

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Core traits and types for ordered stream processing in async Rust.

## Overview

This crate provides the foundational abstractions used throughout the Fluxion ecosystem:

- **`Timestamped` trait**: Temporal ordering for stream items via timestamps
- **`StreamItem<T>`**: Error-aware stream item wrapper
- **Lock utilities**: Safe mutex operations with error propagation

## Key Types

### Timestamp Traits

Fluxion-core provides two traits for temporal ordering:

#### HasTimestamp - Read-Only Access

Minimal trait for types that expose a timestamp value:

```rust
pub trait HasTimestamp {
    type Timestamp: Ord + Copy + Send + Sync + std::fmt::Debug;

    fn timestamp(&self) -> Self::Timestamp;  // Get timestamp for ordering
}
```

Use this when your type only needs to provide a timestamp for ordering (most common case).

#### Timestamped - Full Wrapper Interface

Extends `HasTimestamp` with an `Inner` type and construction methods for wrapper types:

```rust
pub trait Timestamped: HasTimestamp {
    type Inner: Clone;

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self;
    fn with_fresh_timestamp(value: Self::Inner) -> Self;
    fn into_inner(self) -> Self::Inner;
}
```

Use this for wrapper types like `Sequenced<T>` that wrap an inner value with a timestamp.

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
fluxion-core = "0.4.0"
```

## License

Apache-2.0
