# fluxion-core

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Core traits and types for ordered stream processing in async Rust.

## Overview

This crate provides the foundational abstractions used throughout the Fluxion ecosystem:

- **`Ordered` trait**: Temporal ordering for stream items via sequence numbers
- **`OrderedItem<T>`**: Default implementation of `Ordered`
- **Lock utilities**: Safe mutex operations with error propagation

## Key Types

### Ordered Trait

The `Ordered` trait enables temporal ordering guarantees across stream operations:

```rust
pub trait Ordered: Clone {
    type Inner: Clone;

    fn order(&self) -> u64;  // Temporal sequence number
    fn get(&self) -> &Self::Inner;  // Access inner value
    fn with_order(value: Self::Inner, order: u64) -> Self;
}
```

### OrderedItem<T>

A ready-to-use implementation of `Ordered`:

```rust
use fluxion_core::{Ordered, OrderedItem};

let item = OrderedItem::new(42, 1);
assert_eq!(item.order(), 1);
assert_eq!(*item.get(), 42);
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
fluxion-core = "0.2.1"
```

## License

Apache-2.0
