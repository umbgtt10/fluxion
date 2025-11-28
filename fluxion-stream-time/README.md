# fluxion-stream-time

Time-based operators for [fluxion-stream](https://crates.io/crates/fluxion-stream) using chrono timestamps.

This crate provides the `delay` operator and `ChronoTimestamped` wrapper for real-world time-based stream operations.

## Features

- **`ChronoTimestamped<T>`** - Wrapper type with `DateTime<Utc>` timestamps
- **`delay(duration)`** - Delays each emission by a specified duration
- **`ChronoStreamOps`** - Extension trait for `FluxionStream` with chrono-timestamped items

## Usage

```rust
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
use chrono::Duration;

// Create a stream with ChronoTimestamped items
let delayed_stream = FluxionStream::new(source_stream)
    .delay(Duration::milliseconds(100));
```

## Requirements

This crate uses `chrono::Duration` for time operations and `tokio` for async delays.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
