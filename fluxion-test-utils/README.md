## fluxion-test-utils

Shared test utilities and fixtures used across the fluxion workspace. This crate contains helpers for creating sequenced streams, reusable test data, and common assertions to simplify unit and integration tests.

## What you'll find here

- `Sequenced<T>` wrapper for adding temporal ordering to test values
- Test data fixtures: `Person`, `Animal`, `Plant`
- `TestData` enum for diverse test scenarios
- Assertion helpers for stream testing

## Quick example

```rust
use fluxion_test_utils::Sequenced;
use fluxion_stream::FluxionStream;

#[tokio::test]
async fn example() {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let stream = FluxionStream::from_unbounded_receiver(rx);
    
    // Send sequenced values
    tx.send(Sequenced::with_sequence(1, 100)).unwrap();
    tx.send(Sequenced::with_sequence(2, 200)).unwrap();
    tx.send(Sequenced::with_sequence(3, 300)).unwrap();
}
```

## Running tests

```powershell
cargo test --package fluxion-test-utils --all-features --all-targets
```

License

Apache-2.0
