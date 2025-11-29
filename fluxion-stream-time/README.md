# fluxion-stream-time

Time-based operators for [fluxion-stream](https://crates.io/crates/fluxion-stream) using chrono timestamps.

This crate provides the `delay` operator and `ChronoTimestamped` wrapper for real-world time-based stream operations.

## Features

- **`ChronoTimestamped<T>`** - Wrapper type with `DateTime<Utc>` timestamps
- **`delay(duration)`** - Delays each emission by a specified duration
- **`debounce(duration)`** - Emits values only after a quiet period
- **`throttle(duration)`** - Emits a value and then ignores subsequent values for a duration
- **`sample(duration)`** - Emits the most recent value within periodic time intervals
- **`ChronoStreamOps`** - Extension trait for `FluxionStream` with chrono-timestamped items

## Usage

```rust
use fluxion_stream::FluxionStream;
use fluxion_stream_time::{ChronoTimestamped, ChronoStreamOps};
use std::time::Duration;

// Create a stream with ChronoTimestamped items
let delayed_stream = FluxionStream::new(source_stream)
    .delay(Duration::from_millis(100));

// Debounce emissions
let debounced_stream = FluxionStream::new(source_stream)
    .debounce(Duration::from_millis(100));

// Sample emissions
let sampled_stream = FluxionStream::new(source_stream)
    .sample(Duration::from_millis(100));
```

## Requirements

This crate uses `std::time::Duration` for time operations and `tokio` for async delays.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
