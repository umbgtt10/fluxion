# fluxion-test-utils

Shared test utilities and fixtures used across the fluxion workspace. This crate contains helpers for creating timestamped streams, reusable test data, and common assertions to simplify unit and integration tests.

What you'll find here

- Helpers to create timestamped items and channels used by the stream combinators
- Small fixtures for `person`, `animal`, and `plant` test data
- Assertion helpers for deterministic ordering and equality checks

Quick example

```rust
use fluxion_test_utils::timestamped::make_timestamped_stream;
use tokio_stream::StreamExt;

let s = make_timestamped_stream(vec![1, 2, 3]);
let collected: Vec<_> = s.collect().await;
// assert ordering and content
```

Running tests

```powershell
cargo test --package fluxion-test-utils --all-features --all-targets
```

License

MIT OR Apache-2.0
