# fluxion-stream

Stream combinators with ordering guarantees for async Rust.

## Features

- **Temporal Ordering**: Built-in sequence numbering ensures correct event ordering
- **Composable**: Combine streams in various ways while maintaining order guarantees
- **Zero-copy**: Efficient stream processing with minimal allocations
- **Type-safe**: Leverages Rust's type system for compile-time guarantees

## Stream Combinators

- `combine_latest` - Combine multiple streams, emitting when all have published
- `combine_with_previous` - Pair each value with its predecessor
- `with_latest_from` - Combine with the latest value from another stream
- `merge_with` - Merge streams with shared state management
- `take_latest_when` - Conditionally take latest value
- `select_all_ordered` - Multiplex streams with ordering

## Core Types

- `Timestamped<T>` - Wrapper that adds logical timestamps for temporal ordering
- `timestamped_channel` - Channels that automatically timestamp messages
- `CompareByInner` - Trait for dual-ordering strategies (structural vs temporal)

## Example

```rust
use fluxion_stream::{timestamped_channel::unbounded_channel, combine_latest::CombineLatestExt};
use tokio_stream::wrappers::UnboundedReceiverStream;

let (tx1, rx1) = unbounded_channel();
let (tx2, rx2) = unbounded_channel();

let stream1 = UnboundedReceiverStream::new(rx1.into_inner());
let stream2 = UnboundedReceiverStream::new(rx2.into_inner());

let combined = stream1.combine_latest(vec![stream2], |_| true);
```

## License

MIT OR Apache-2.0
