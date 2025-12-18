# fluxion-stream-time

Time-based operators for [fluxion-stream](https://crates.io/crates/fluxion-stream) using real-world time.

This crate provides specialized time-based operators (`delay`, `debounce`, `throttle`, `sample`, `timeout`) and the `InstantTimestamped<T>` wrapper for working with monotonic timestamps using `std::time::Instant`.

## Why This Crate Exists

Fluxion's core design is **timestamp-agnostic**: operators in `fluxion-stream` work with any type implementing the `HasTimestamp` trait, regardless of the underlying timestamp representation (u64 counters, DateTime, custom types, etc.). This flexibility is a core strength.

However, **time-based operators** like `delay`, `debounce`, `throttle`, and `timeout` inherently need to perform **arithmetic on timestamps** (e.g., "add 100ms to this timestamp"). This requires a specific timestamp type that supports duration arithmetic.

### The Two Timestamp Approaches

**1. Counter-Based (fluxion-test-utils)**
- **Type**: `Sequenced<T>` with `u64` timestamps
- **Use case**: Testing, simulation, reproducible scenarios
- **Operators**: All core operators in `fluxion-stream`
- **Advantage**: Deterministic, no time dependencies, fast

**2. Real-Time Based (fluxion-stream-time)**
- **Type**: `InstantTimestamped<T>` with `std::time::Instant` timestamps
- **Use case**: Production systems, real-world scheduling, monotonic time
- **Operators**: Time-based operators (`delay`, `debounce`, `throttle`, `sample`, `timeout`)
- **Advantage**: Monotonic time, duration arithmetic, no system clock dependencies

### Why Not Merge These?

Keeping `fluxion-stream` timestamp-agnostic means:
- ✅ **Minimal dependencies**: Only `std` and `tokio` for time-based operators
- ✅ **Flexible timestamp types**: You can use custom timestamp representations
- ✅ **Faster compile times**: Time operators are optional and lightweight
- ✅ **Testing independence**: Counter-based timestamps for deterministic tests

## Features

- **`InstantTimestamped<T>`** - Wrapper type with `std::time::Instant` timestamps
- **`delay(duration)`** - Delays each emission by a specified duration
- **`debounce(duration)`** - Emits values only after a quiet period
- **`throttle(duration)`** - Emits a value and then ignores subsequent values for a duration
- **`sample(duration)`** - Emits the most recent value within periodic time intervals
- **`timeout(duration)`** - Errors if no emission within duration
- **`InstantStreamOps`** - Extension trait for `FluxionStream` with Instant-timestamped items

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
let delayed = stream.delay(Duration::from_millis(100));
```

- Each item delayed independently
- Errors pass through immediately (no delay)
- Preserves temporal ordering
- **Use when**: Adding artificial delays, scheduling emissions

#### `debounce`
**Emits only after a period of inactivity (trailing)**

```rust
let debounced = stream.debounce(Duration::from_millis(500));
```

- Emits latest value after quiet period
- Timer resets on each new value
- Pending value emitted when stream ends
- Errors pass through immediately
- **Use when**: Search-as-you-type, button debouncing, rate limiting user actions

#### `throttle`
**Rate-limits emissions (leading)**

```rust
let throttled = stream.throttle(Duration::from_millis(100));
```

- Emits first value immediately
- Ignores subsequent values for duration
- Then accepts next value and repeats
- Errors pass through immediately
- **Use when**: Scroll/resize handlers, API rate limiting, UI event throttling

#### `sample`
**Samples stream at periodic intervals**

```rust
let sampled = stream.sample(Duration::from_millis(100));
```

- Emits most recent value within each interval
- No emission if no value in interval
- Errors pass through immediately
- **Use when**: Downsampling sensors, periodic snapshots, metrics aggregation

#### `timeout`
**Errors if no emission within duration**

```rust
let with_timeout = stream.timeout(Duration::from_secs(30));
```

- Monitors time between emissions
- Emits `FluxionError::StreamProcessingError("Timeout")` if exceeded
- Timer resets on each emission (value or error)
- Stream terminates on timeout
- **Use when**: Watchdog timers, network reliability, health checks

## Seamless Operator Chaining

**Key insight**: Operators from both crates chain seamlessly because they all work with streams where `S: Stream<Item = StreamItem<T>>` and `T: HasTimestamp`.

The only requirement is that your stream items implement `HasTimestamp` with a compatible timestamp type:
- **Core operators** (`map_ordered`, `filter_ordered`, `combine_latest`, etc.) work with **any** timestamp type
- **Time operators** (`delay`, `debounce`, etc.) **only** work with `DateTime<Utc>` timestamps

### Example: Mixing Operators

```rust
use fluxion_stream::{IntoFluxionStream, FilterOrderedExt, MapOrderedExt, DistinctUntilChangedExt};
use fluxion_stream_time::{InstantTimestamped, DelayExt, DebounceExt};
use std::time::Duration;

// Start with time-based stream
let stream = source_stream
    // Time operator (requires std::time::Instant)
    .debounce(Duration::from_millis(100))

    // Core operators work seamlessly
    .filter_ordered(|item| *item > 50)
    .map_ordered(|item| item * 2)

    // Back to time operator
    .delay(Duration::from_millis(50))

    // More core operators
    .distinct_until_changed();
```

**This works because:**
1. `debounce` returns `impl Stream<Item = StreamItem<InstantTimestamped<T>>>`
2. Core operators like `filter_ordered` accept **any** stream where items implement `HasTimestamp`
3. The timestamp type (`std::time::Instant`) is preserved through the chain
4. `delay` can then use it again because the type is still `InstantTimestamped<T>`

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

## Usage

```rust
use fluxion_stream_time::{InstantTimestamped, DelayExt, DebounceExt, SampleExt};
use std::time::Duration;

// Delay all emissions by 100ms
let delayed_stream = source_stream
    .delay(Duration::from_millis(100));

// Debounce emissions
let debounced_stream = source_stream
    .debounce(Duration::from_millis(100));

// Sample emissions
let sampled_stream = source_stream
    .sample(Duration::from_millis(100));
```

## InstantTimestamped vs Sequenced

| Feature | `Sequenced<T>` (test-utils) | `InstantTimestamped<T>` (stream-time) |
|---------|----------------------------|-------------------------------------|
| **Timestamp Type** | `u64` (counter) | `std::time::Instant` |
| **Crate** | `fluxion-test-utils` | `fluxion-stream-time` |
| **Use Case** | Testing, simulation | Production, real time |
| **Time Operators** | ❌ No | ✅ Yes |
| **Core Operators** | ✅ Yes | ✅ Yes |
| **Deterministic** | ✅ Yes | ❌ No (monotonic) |
| **Duration Math** | ❌ No | ✅ Yes |
| **Dependencies** | None | `std`, `tokio` |

## Requirements

This crate uses `std::time::{Duration, Instant}` for time operations and `tokio` for async delays. No external time dependencies required.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
