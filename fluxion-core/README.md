# fluxion-core

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Core traits and types for ordered stream processing in async Rust.

## Overview

This crate provides the foundational abstractions used throughout the Fluxion ecosystem:

- **`Timestamped` trait**: Temporal ordering for stream items via timestamps
- **`StreamItem<T>`**: Error-aware stream item wrapper (`Value` | `Error`)
- **`FluxionSubject<T>`**: Hot, multi-subscriber broadcast subject
- **`FluxionError`**: Unified error type for stream operations
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

### FluxionSubject<T>

A hot, multi-subscriber broadcast subject for reactive programming patterns:

```rust
use fluxion_core::{FluxionSubject, StreamItem};
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let subject = FluxionSubject::new();

    // Subscribe before sending - hot subject, no replay
    let mut stream1 = subject.subscribe();
    let mut stream2 = subject.subscribe();

    // Send to all subscribers
    subject.send(StreamItem::Value(42)).unwrap();

    // Both subscribers receive the value
    assert_eq!(stream1.next().await.unwrap().unwrap(), 42);
    assert_eq!(stream2.next().await.unwrap().unwrap(), 42);
}
```

**Key Characteristics:**

- **Hot**: Late subscribers only receive items sent *after* they subscribe (no replay buffer)
- **Multi-subscriber**: Broadcasts each item to all active subscribers simultaneously
- **Thread-safe**: Uses `Arc<Mutex<>>` internally - cheap to clone, safe to send across threads
- **Automatic cleanup**: Dead subscribers are removed on next `send()` (no memory leaks)
- **Unbounded**: Uses unbounded mpsc channels (no backpressure)

**Subject Lifecycle:**

```rust
let subject = FluxionSubject::new();

// Clone shares the same subject state
let subject_clone = subject.clone();

// Subscribe on any clone
let stream = subject_clone.subscribe();

// Send on any clone - all subscribers receive it
subject.send(StreamItem::Value(1)).unwrap();

// Error terminates all subscribers
subject.error(FluxionError::stream_error("failed"));

// Explicit close completes all subscribers
subject.close();
```

**Thread Safety:**

```rust
let subject = FluxionSubject::new();

// Safe to share across async tasks
tokio::spawn({
    let subject = subject.clone();
    async move {
        subject.send(StreamItem::Value(1)).unwrap();
    }
});

tokio::spawn({
    let subject = subject.clone();
    async move {
        subject.subscribe();
    }
});
```

**Common Patterns:**

1. **Event Bus**: Broadcast domain events to multiple handlers
2. **State Updates**: Notify observers of state changes
3. **Message Fanout**: Distribute work to multiple consumers
4. **Test Doubles**: Injectable subjects for testing reactive flows

**When to Use:**
- ✅ Multiple subscribers need the same stream
- ✅ Subscribers can join/leave dynamically
- ✅ No replay needed (hot semantics)
- ✅ Unbounded buffers acceptable

**When NOT to Use:**
- ❌ Need cold semantics (replay to new subscribers)
- ❌ Need backpressure (bounded channels)
- ❌ Single subscriber (use channels directly)
- ❌ Persistent event log (use actual event store)

### StreamItem<T>

Error-aware wrapper for stream values:

```rust
pub enum StreamItem<T> {
    Value(T),
    Error(FluxionError),
}
```

Enables error propagation through operator chains without terminating the stream. See the [Error Handling Guide](../docs/ERROR-HANDLING.md) for details.

## Architecture Notes

### Why FluxionSubject Uses Arc<Mutex<>>

The subject's design requires interior mutability with shared ownership:

**Operations requiring mutation:**
- `send()` - broadcasts to all subscribers AND removes dead ones
- `subscribe()` - adds new subscriber to the list
- `close()` - sets closed flag and clears subscribers

**Why Arc:**
- Subject is `Clone` - multiple handles can exist
- All clones share the same subscriber list and state
- Enables passing to different async tasks

**Why Mutex:**
- Multiple threads/tasks can call operations concurrently
- Prevents data races on the subscriber `Vec`
- Ensures consistent state across `send()`/`subscribe()`/`close()`

**Alternative considered:** `&mut self` methods would prevent cloning and multi-task sharing - defeats the purpose of a broadcast subject.

### Dead Subscriber Cleanup

Subscribers are automatically cleaned up during `send()`:

```rust
// For each send, remove disconnected subscribers
for tx in state.senders.drain(..) {
    if tx.unbounded_send(item.clone()).is_ok() {
        next_senders.push(tx);  // Keep alive
    }
    // Dead subscribers dropped here
}
```

This prevents memory leaks when subscribers drop their streams without explicitly unsubscribing.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
fluxion-core = "0.6.0"
```

## License

Apache-2.0
