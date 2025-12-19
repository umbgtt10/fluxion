# fluxion-stream-time

Runtime-agnostic time-based operators for [fluxion-stream](https://crates.io/crates/fluxion-stream).

This crate provides specialized time-based operators (`delay`, `debounce`, `throttle`, `sample`, `timeout`) that work with any async runtime through the `Timer` trait abstraction. The `InstantTimestamped<T, TM>` wrapper provides timestamping with the runtime's monotonic instant type.

## Runtime Support

fluxion-stream-time supports multiple async runtimes through feature flags:

- **`time-tokio`** (default) - Tokio runtime with `TokioTimer`
- **`time-wasm`** - WebAssembly with `WasmTimer` (Node.js and browser)
- **`time-async-std`** - async-std runtime (planned)
- **`time-smol`** - smol runtime (planned)

All operators are fully runtime-agnostic thanks to the `Timer` trait abstraction.

## Why This Crate Exists

Fluxion's core design is **timestamp-agnostic**: operators in `fluxion-stream` work with any type implementing the `HasTimestamp` trait, regardless of the underlying timestamp representation (u64 counters, DateTime, custom types, etc.). This flexibility is a core strength.

However, **time-based operators** like `delay`, `debounce`, `throttle`, and `timeout` inherently need to:
1. **Perform arithmetic on timestamps** (e.g., "add 100ms to this timestamp")
2. **Sleep asynchronously** for specific durations
3. **Support different async runtimes** (Tokio, async-std, smol)

The `Timer` trait abstraction solves all three requirements, enabling operators that work with any runtime while maintaining zero-cost performance.

### The Two Timestamp Approaches

**1. Counter-Based (fluxion-test-utils)**
- **Type**: `Sequenced<T>` with `u64` timestamps
- **Use case**: Testing, simulation, reproducible scenarios
- **Operators**: All core operators in `fluxion-stream`
- **Advantage**: Deterministic, no time dependencies, fast

**2. Real-Time Based (fluxion-stream-time)**
- **Type**: `InstantTimestamped<T, TM: Timer>` with runtime's `Instant` type
- **Use case**: Production systems, real-world scheduling, monotonic time
- **Operators**: Time-based operators (`delay`, `debounce`, `throttle`, `sample`, `timeout`)
- **Advantage**: Monotonic time, duration arithmetic, runtime flexibility

### Why Not Merge These?

Keeping `fluxion-stream` timestamp-agnostic means:
- ✅ **Minimal dependencies**: Core stream operators have no runtime dependencies
- ✅ **Flexible timestamp types**: You can use custom timestamp representations
- ✅ **Faster compile times**: Time operators are optional and lightweight
- ✅ **Testing independence**: Counter-based timestamps for deterministic tests
- ✅ **Runtime flexibility**: Choose your async runtime via feature flags

## Features

### Core Types
- **`Timer` trait** - Runtime-agnostic timer abstraction with `sleep_future()` and `now()`
- **`TokioTimer`** - Zero-cost Tokio implementation (when `time-tokio` enabled)
- **`InstantTimestamped<T, TM>`** - Generic wrapper with timer's `Instant` type
- **`TokioTimestamped<T>`** - Type alias for `InstantTimestamped<T, TokioTimer>`

### Operators
- **`delay(duration, timer)`** - Delays each emission by a specified duration
- **`debounce(duration, timer)`** - Emits values only after a quiet period
- **`throttle(duration, timer)`** - Emits a value and then ignores subsequent values for a duration
- **`sample(duration, timer)`** - Emits the most recent value within periodic time intervals
- **`timeout(duration, timer)`** - Errors if no emission within duration

## Quick Reference Table

| Operator | Purpose | Behavior | Use Case |
|----------|---------|----------|----------|
| [`delay`](#delay) | Time-shift emissions | Delays each item by duration, errors pass through | Artificial delays, scheduling |
| [`debounce`](#debounce) | Trailing debounce | Emits after quiet period, resets on new value | Search input, button debouncing |
| [`throttle`](#throttle) | Leading throttle | Emits first, ignores subsequent for duration | Rate limiting, scroll/resize handlers |
| [`sample`](#sample) | Periodic sampling | Emits latest value at intervals | Downsampling high-frequency streams |
| [`timeout`](#timeout) | Watchdog timer | Errors if no emission within duration | Network reliability, health checks |

### Operator Details

#### `delay`
**Delays each emission by a specified duration**

```rust
let timer = TokioTimer;
let delayed = stream.delay(Duration::from_millis(100), timer);
```

- Each item delayed independently
- Errors pass through immediately (no delay)
- Preserves temporal ordering
- **Use when**: Adding artificial delays, scheduling emissions

#### `debounce`
**Emits only after a period of inactivity (trailing)**

```rust
let timer = TokioTimer;
let debounced = stream.debounce(Duration::from_millis(500), timer);
```

- Emits latest value after quiet period
- Timer resets on each new value
- Pending value emitted when stream ends
- Errors pass through immediately
- **Use when**: Search-as-you-type, button debouncing, rate limiting user actions

#### `throttle`
**Rate-limits emissions (leading)**

```rust
let timer = TokioTimer;
let throttled = stream.throttle(Duration::from_millis(100), timer);
```

- Emits first value immediately
- Ignores subsequent values for duration
- Then accepts next value and repeats
- Errors pass through immediately
- **Use when**: Scroll/resize handlers, API rate limiting, UI event throttling

#### `sample`
**Samples stream at periodic intervals**

```rust
let timer = TokioTimer;
let sampled = stream.sample(Duration::from_millis(100), timer);
```

- Emits most recent value within each interval
- No emission if no value in interval
- Errors pass through immediately
- **Use when**: Downsampling sensors, periodic snapshots, metrics aggregation

#### `timeout`
**Errors if no emission within duration**

```rust
let timer = TokioTimer;
let with_timeout = stream.timeout(Duration::from_secs(30), timer);
```

- Monitors time between emissions
- Emits `FluxionError::TimeoutError("Timeout")` if exceeded
- Timer resets on each emission (value or error)
- Stream terminates on timeout
- **Use when**: Watchdog timers, network reliability, health checks

## Quick Start Example

```rust
use fluxion_stream_time::prelude::*;
use fluxion_stream_time::{TokioTimer, TokioTimestamped};
use fluxion_stream_time::timer::Timer;
use fluxion_core::StreamItem;
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel();
    let timer = TokioTimer;

    // Create timestamped stream with runtime-aware delays
    let stream = UnboundedReceiverStream::new(rx)
        .map(StreamItem::Value)
        .debounce(Duration::from_millis(100), timer.clone())
        .throttle(Duration::from_millis(200), timer.clone());

    // Send timestamped data
    tx.send(TokioTimestamped::new(42, timer.now())).unwrap();
    tx.send(TokioTimestamped::new(100, timer.now())).unwrap();
}
```

## Seamless Operator Chaining

**Key insight**: Operators from both crates chain seamlessly because they all work with streams where `S: Stream<Item = StreamItem<T>>` and `T: HasTimestamp`.

The only requirement is that your stream items implement `HasTimestamp` with a compatible timestamp type:
- **Core operators** (`map_ordered`, `filter_ordered`, `combine_latest`, etc.) work with **any** timestamp type
- **Time operators** (`delay`, `debounce`, etc.) work with `InstantTimestamped<T, TM: Timer>` types

### Example: Mixing Operators

```rust
use fluxion_stream::{IntoFluxionStream, FilterOrderedExt, MapOrderedExt, DistinctUntilChangedExt};
use fluxion_stream_time::{TokioTimer, TokioTimestamped, DelayExt, DebounceExt};
use std::time::Duration;

let timer = TokioTimer;

// Start with time-based stream
let stream = source_stream
    // Time operator (requires Timer)
    .debounce(Duration::from_millis(100), timer.clone())

    // Core operators work seamlessly
    .filter_ordered(|item| *item > 50)
    .map_ordered(|item| item * 2)

    // Back to time operator
    .delay(Duration::from_millis(50), timer.clone())

    // More core operators
    .distinct_until_changed();
```

**This works because:**
1. `debounce` returns `impl Stream<Item = StreamItem<TokioTimestamped<T>>>`
2. Core operators like `filter_ordered` accept **any** stream where items implement `HasTimestamp`
3. The timestamp type (Timer's `Instant`) is preserved through the chain
4. `delay` can then use it again because the type is still `InstantTimestamped<T, TM>`

### When to Use Each Crate

**Use `fluxion-stream` (core operators) when:**
- You need ordering, combining, filtering, transformation
- You want timestamp-agnostic code
- You're testing with counter-based timestamps (`Sequenced<T>`)

**Use `fluxion-stream-time` (time operators) when:**
- You need real-world time-based behavior (`delay`, `debounce`, etc.)
- You're working with production event streams requiring monotonic time
- You need duration arithmetic without system clock dependencies

**Use both together when:**
- You need time-based rate limiting **plus** complex stream transformations
- You want to debounce user input, then combine it with other streams
- You're building real-world reactive systems with temporal constraints

## Usage with Different Runtimes

### Tokio (default)

```rust
use fluxion_stream_time::{TokioTimer, TokioTimestamped, DelayExt, DebounceExt};
use fluxion_stream_time::timer::Timer;
use std::time::Duration;

let timer = TokioTimer;

// Delay all emissions by 100ms
let delayed_stream = source_stream
    .delay(Duration::from_millis(100), timer.clone());

// Debounce emissions
let debounced_stream = source_stream
    .debounce(Duration::from_millis(100), timer.clone());

// Create timestamped values
let timestamped = TokioTimestamped::new(my_value, timer.now());
```

### async-std (planned)

```rust
use fluxion_stream_time::{AsyncStdTimer, DelayExt};
use std::time::Duration;

let timer = AsyncStdTimer;
let delayed = source_stream.delay(Duration::from_millis(100), timer);
```

### smol (planned)

```rust
use fluxion_stream_time::{SmolTimer, DelayExt};
use std::time::Duration;

let timer = SmolTimer;
let delayed = source_stream.delay(Duration::from_millis(100), timer);
```

### WASM (WebAssembly)

```rust
use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
use fluxion_stream_time::{DelayExt, DebounceExt, InstantTimestamped};
use fluxion_stream_time::timer::Timer;
use std::time::Duration;

let timer = WasmTimer::new();

// Delay all emissions by 100ms
let delayed_stream = source_stream
    .delay(Duration::from_millis(100), timer.clone());

// Debounce emissions
let debounced_stream = source_stream
    .debounce(Duration::from_millis(100), timer.clone());

// Create timestamped values
let timestamped = InstantTimestamped::new(my_value, timer.now());
```

**WASM Notes:**
- Uses `gloo-timers` for async sleep (compatible with Node.js and browsers)
- Custom `WasmInstant` based on `js-sys::Date.now()` for monotonic time
- Tests run with `wasm-pack test --node` or `--headless --chrome`
- 5 comprehensive tests validate all time-based operators in WASM environments

## Timer Trait Implementation

To add support for a custom runtime, implement the `Timer` trait:

```rust
use fluxion_stream_time::timer::Timer;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct MyCustomTimer;

impl Timer for MyCustomTimer {
    type Sleep = MyRuntimeSleep;
    type Instant = Instant;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep {
        my_runtime::sleep(duration)
    }

    fn now(&self) -> Self::Instant {
        Instant::now()
    }
}
```

## InstantTimestamped vs Sequenced

| Feature | `Sequenced<T>` (test-utils) | `InstantTimestamped<T, TM>` (stream-time) |
|---------|----------------------------|------------------------------------------|
| **Timestamp Type** | `u64` (counter) | `TM::Instant` (runtime's instant) |
| **Crate** | `fluxion-test-utils` | `fluxion-stream-time` |
| **Use Case** | Testing, simulation | Production, real time |
| **Time Operators** | ❌ No | ✅ Yes |
| **Core Operators** | ✅ Yes | ✅ Yes |
| **Deterministic** | ✅ Yes | ❌ No (monotonic) |
| **Duration Math** | ❌ No | ✅ Yes |
| **Runtime Support** | N/A | Tokio, WASM (async-std, smol planned) |

## Future Platform Support

### no_std Feasibility

The `Timer` trait abstraction makes **no_std support architecturally feasible**. The trait design deliberately avoids std-specific dependencies, enabling potential embedded system support.

#### What's Already Compatible

- ✅ **Timer trait** - Uses only `core::future::Future` and `core::time::Duration`
- ✅ **Generic operators** - Work with any Timer implementation
- ✅ **Pin projection** - pin-project supports no_std
- ✅ **Type-safe instant arithmetic** - Generic over `Timer::Instant`

#### Challenges for no_std

1. **`std::time::Instant` unavailable**
   - Embedded systems need platform-specific tick counters
   - Solution: Custom `Instant` type based on hardware timers

2. **`alloc` requirement**
   - Operators use `Box::pin()` for stream composition
   - Requires heap allocation (minimal: one box per operator)
   - Some embedded environments may not have allocators

3. **Async runtime dependency**
   - Tokio requires std
   - Potential alternatives: **Embassy** (async for embedded), **RTIC** (real-time)
   - Would require separate Timer implementations per runtime

#### Implementation Paths

**Embassy Support** (recommended for async embedded):
```rust
// Future: EmbassyTimer implementation
use embassy_time::{Duration, Instant, Timer as EmbassyDelay};

impl Timer for EmbassyTimer {
    type Sleep = EmbassyDelay;
    type Instant = Instant;  // u64 tick counter
    // ...
}
```

**Bare Metal** (custom hardware timers):
- Custom tick-based Instant type
- Manual future polling against hardware timer registers
- More complex but possible with current architecture

#### Trade-offs

The Timer abstraction **enables** no_std support without forcing it:
- std users get ergonomic APIs (Tokio, async-std)
- Embedded users can implement custom timers when needed
- No compromise in type safety or performance for either

**Status**: Architecturally sound, implementation work required. The generic design means no_std support won't break existing std code.

### WASM Support

WASM support is **fully implemented** via `WasmTimer` using `gloo-timers` and `js-sys`. The Timer trait abstraction enabled this with zero operator changes.

**Implementation Details:**
- **`WasmTimer`** - Zero-cost WASM implementation using `gloo_timers::future::sleep`
- **`WasmInstant`** - Custom instant type based on `js-sys::Date.now()` (returns milliseconds since epoch)
- **Arithmetic support** - Implements `Add<Duration>`, `Sub<Duration>`, and `Sub<Self>` for duration calculations
- **Runtime compatibility** - Works in both Node.js and browser environments

**Testing:**
- Integration tests in `tests/wasm/single_threaded/`
- Tests use real delays with `gloo_timers::future::sleep` (no time control like Tokio)
- Run with `wasm-pack test --node --features time-wasm`
- See [WASM_TESTING.md](WASM_TESTING.md) for complete testing guide

**Platform Support:**
- ✅ Node.js (v14+)
- ✅ Modern browsers (Chrome, Firefox, Safari, Edge)
- ✅ GitHub Actions CI with wasm-pack
- ✅ Single-threaded execution model

## Requirements

- Rust 1.70+
- Choose runtime via feature flags (`time-tokio` enabled by default)
- No external time dependencies beyond the chosen async runtime

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
