# fluxion-stream# fluxion-stream



Stream combinators for async Rust with strong temporal-ordering guarantees. This crate provides a set of composable operators and lightweight timestamping utilities designed for correctness and performance in event-driven systems.Stream combinators for async Rust with strong temporal-ordering guarantees. This crate provides a set of composable operators and lightweight timestamping utilities designed for correctness and performance in event-driven systems.



FeaturesKey features



- Temporal ordering via `Timestamped<T>` and sequence numbers- Temporal ordering via Timestamped<T> and sequence numbers

- Composable operators: `combine_latest`, `with_latest_from`, `merge_with`, `take_latest_when`, `select_all_ordered`, and more- Composable operators: combine_latest, with_latest_from, merge_with, take_latest_when, and more

- Efficient implementation with minimal allocations- Efficient implementation with minimal allocations



Core modulesCore modules



- `timestamped` — `Timestamped<T>` wrapper and helpers- 	imestamped  Timestamped<T> wrapper and helpers

- `timestamped_channel` — channels that assign timestamps automatically- 	imestamped_channel  channels that assign timestamps automatically

- `combine_latest`, `merge_with`, `combine_with_previous`, `select_all_ordered`, `take_latest_when`- combine_latest  combine multiple streams while preserving order guarantees

- merge_with, combine_with_previous, select_all_ordered, 	ake_latest_when

Quick example

Quick example

```rust

use fluxion_stream::timestamped::Timestamped;`

use tokio_stream::StreamExt;ust

use fluxion_stream::timestamped::Timestamped;

// `stream` is a Stream of Timestamped itemsuse tokio_stream::StreamExt;

// create streams and combine (see crate docs for detailed examples)

```// create streams and combine (see crate docs for detailed examples)

`

Running tests

Running tests

```powershell

cargo test --package fluxion-stream --all-features --all-targets`powershell

```cargo test --package fluxion-stream --all-features --all-targets

`

Documentation

Documentation

```powershell

cargo doc --package fluxion-stream --no-deps --openRun cargo doc --package fluxion-stream --no-deps --open to view generated docs locally.

```

License

License

MIT OR Apache-2.0

MIT OR Apache-2.0
