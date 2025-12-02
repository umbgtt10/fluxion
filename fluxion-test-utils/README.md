## fluxion-test-utils

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Shared test utilities and fixtures used across the fluxion workspace. This crate provides helpers for creating sequenced streams with deterministic timestamps, reusable test data, and common assertions to simplify unit and integration tests.

## Features

- **`Sequenced<T>`** - Counter-based timestamp wrapper for deterministic testing
- **Test Data Fixtures** - Pre-defined `Person`, `Animal`, `Plant` types
- **`TestData` Enum** - Unified enum for diverse test scenarios
- **Assertion Helpers** - Stream testing utilities
- **Error Injection** - `ErrorInjectingStream` for testing error handling

## Quick Reference

| Utility | Purpose | Use Case |
|---------|---------|----------|
| `Sequenced<T>` | Counter-based timestamps (u64) | Deterministic ordering in tests |
| `Sequenced::new(value)` | Auto-assigned sequence number | Quick test values |
| `Sequenced::with_timestamp(value, ts)` | Explicit timestamp | Control ordering precisely |
| `test_channel()` | Create test channel | Simplified test setup |
| `unwrap_stream()` | Extract values from StreamItem | Clean test assertions |
| `assert_no_element_emitted()` | Verify stream silence | Timeout testing |

## Why Sequenced<T>?

Unlike `ChronoTimestamped<T>` in `fluxion-stream-time` (which uses real wall-clock time), `Sequenced<T>` provides:

- ✅ **Deterministic behavior**: Tests always produce same results
- ✅ **No time dependencies**: No `tokio::time::sleep()` needed
- ✅ **Explicit control**: Set exact timestamps for scenarios
- ✅ **Fast execution**: No actual delays
- ✅ **Reproducible bugs**: Same sequence every run

Perfect for unit tests, integration tests, and CI environments.

## Usage Examples

### Basic Sequencing

```rust
use fluxion_test_utils::Sequenced;
use fluxion_stream::{FluxionStream, IntoFluxionStream};

#[tokio::test]
async fn example() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let stream = rx.into_fluxion_stream();

    // Auto-assigned sequence numbers (0, 1, 2, ...)
    tx.send(Sequenced::new(42)).unwrap();
    tx.send(Sequenced::new(100)).unwrap();

    // Explicit timestamps for precise control
    tx.send(Sequenced::with_timestamp(200, 5)).unwrap();
}
```

### Using Test Fixtures

```rust
use fluxion_test_utils::test_data::{person_alice, person_bob, animal_cat};

#[test]
fn test_with_fixtures() {
    let alice = person_alice(); // Person { id: 1, name: "Alice" }
    let bob = person_bob();     // Person { id: 2, name: "Bob" }
    let cat = animal_cat();     // Animal { id: 1, species: "Cat" }

    assert_eq!(alice.name, "Alice");
}
```

### Helper Functions

```rust
use fluxion_test_utils::{test_channel, unwrap_stream};
use fluxion_stream::FluxionStream;

#[tokio::test]
async fn test_with_helpers() {
    // Simplified channel creation
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    tx.send(Sequenced::new(42)).unwrap();

    // Extract values easily
    let values = unwrap_stream(stream, 1).await;
    assert_eq!(values, vec![42]);
}
```

### Error Injection Testing

```rust
use fluxion_test_utils::ErrorInjectingStream;
use fluxion_core::{StreamItem, FluxionError};
use futures::stream;

#[tokio::test]
async fn test_error_handling() {
    let base = stream::iter(vec![
        StreamItem::Value(1),
        StreamItem::Value(2),
    ]);

    // Inject error at position 1
    let stream_with_error = ErrorInjectingStream::new(
        base,
        1,
        FluxionError::custom("Test error")
    );

    // Test your error handling logic
}
```

## Sequenced vs ChronoTimestamped

| Feature | `Sequenced<T>` (this crate) | `ChronoTimestamped<T>` (stream-time) |
|---------|----------------------------|-------------------------------------|
| **Timestamp Type** | `u64` (counter) | `DateTime<Utc>` |
| **Purpose** | Testing, simulation | Production, real time |
| **Deterministic** | ✅ Always | ❌ Wall-clock dependent |
| **Time Operators** | ❌ No | ✅ Yes (`delay`, `debounce`, etc.) |
| **Core Operators** | ✅ Yes (all) | ✅ Yes (all) |
| **Speed** | ⚡ Instant | 🐌 Real delays |
| **Dependencies** | None | `chrono`, `tokio::time` |
| **Best For** | Unit/integration tests | Production systems |

## API Overview

### Sequenced<T>

```rust
impl<T> Sequenced<T> {
    // Create with auto-assigned sequence
    pub fn new(value: T) -> Self;

    // Create with explicit timestamp
    pub fn with_timestamp(value: T, timestamp: u64) -> Self;

    // Extract inner value
    pub fn into_inner(self) -> T;

    // Access inner value (via public field)
    pub value: T;
}
```

### Test Fixtures

Pre-defined test data available in `test_data` module:

```rust
// People
person_alice()  // Person { id: 1, name: "Alice" }
person_bob()    // Person { id: 2, name: "Bob" }
person_carol()  // Person { id: 3, name: "Carol" }

// Animals
animal_cat()    // Animal { id: 1, species: "Cat" }
animal_dog()    // Animal { id: 2, species: "Dog" }

// Plants
plant_rose()    // Plant { id: 1, name: "Rose" }
plant_tulip()   // Plant { id: 2, name: "Tulip" }
```

### Helper Functions

```rust
// Create test channel with FluxionStream
pub fn test_channel<T>() -> (UnboundedSender<T>, FluxionStream<...>);

// Extract values from stream
pub async fn unwrap_stream<T>(stream: S, count: usize) -> Vec<T>;

// Extract single value
pub async fn unwrap_value<T>(stream: S) -> T;

// Assert no emission within timeout
pub async fn assert_no_element_emitted<S>(stream: &mut S, timeout_ms: u64);

// Assert stream has ended
pub async fn assert_stream_ended<S>(stream: &mut S);
```

## Running tests

```powershell
cargo test --package fluxion-test-utils --all-features --all-targets
```

## See Also

- **[fluxion-stream-time](../fluxion-stream-time/README.md)** - Time-based operators with `ChronoTimestamped<T>`
- **[Fluxion Documentation](../README.md)** - Main project documentation

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
