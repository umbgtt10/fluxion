# fluxion-stream

Stream combinators for async Rust with strong temporal-ordering guarantees. This crate provides composable operators and lightweight sequencing utilities designed for correctness and performance in event-driven systems.

Key features

- Temporal ordering via `Sequenced<T>` and sequence numbers
- Composable operators: `combine_latest`, `with_latest_from`, `merge_with`, `take_latest_when`, `ordered_merge`, and more
- Efficient implementation with minimal allocations

Core modules

- `sequenced` — `Sequenced<T>` wrapper and helpers
- `sequenced_channel` — channels that assign sequence numbers automatically (test utilities)
- Operator modules: `combine_latest`, `merge_with`, `combine_with_previous`, `ordered_merge`, `take_latest_when`, `with_latest_from`

Quick example

```rust
use fluxion_stream::sequenced::Sequenced;
use tokio_stream::StreamExt;

// `stream` is a Stream of Sequenced items
// Example: map a sequenced stream to its inner values
// let values = stream.map(|s: Sequenced<_>| s.value).collect::<Vec<_>>().await;
```

Running tests

```powershell
cargo test --package fluxion-stream --all-features --all-targets
```

Documentation

```powershell
cargo doc --package fluxion-stream --no-deps --open
```

License

MIT OR Apache-2.0
