# fluxion-exec

Async stream subscribers and execution utilities for fluxion.

## Features

- **Async Subscribers**: Execute async handlers for each stream item
- **Cancellation Support**: Built-in cancellation token support
- **Error Handling**: Customizable error callbacks
- **Execution Strategies**: Sequential and parallel processing modes

## Subscribers

### `subscribe_async`
Sequential processing of stream items with async handlers.

```rust
use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use tokio_stream::StreamExt;

stream
    .subscribe_async(
        |item, _ctx| async move {
            process(item).await
        },
        None,  // Optional cancellation token
        Some(|err| eprintln!("Error: {:?}", err))
    )
    .await;
```

### `subscribe_latest_async`
Parallel processing with automatic cancellation of outdated work.

```rust
use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;

stream
    .subscribe_latest_async(
        |item, ctx| async move {
            long_running_task(item, ctx).await
        },
        None,
        Some(|err| eprintln!("Error: {:?}", err))
    )
    .await;
```

## Use Cases

- Processing stream events with async I/O
- Database operations triggered by stream events
- API calls based on stream data
- Background task execution with cancellation

## License

Apache-2.0
