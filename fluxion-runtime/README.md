# fluxion-runtime

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Runtime abstraction layer enabling Fluxion to work across multiple async runtimes and platforms (Tokio, smol, async-std, WASM, Embassy).

## Overview

`fluxion-runtime` provides the core abstractions that allow Fluxion operators to work seamlessly across different async runtimes without code changes. It includes:

- **`Timer` trait** - Runtime-agnostic time abstraction for sleep and instant operations
- **`FluxionMutex` trait** - Mutex abstraction supporting both thread-safe (Arc<Mutex>) and single-threaded (Rc<RefCell>) contexts
- **Runtime implementations** - Concrete timer implementations for 5 different runtimes

## Supported Runtimes

| Runtime | Feature Flag | Platform Support | Thread Safety |
|---------|-------------|------------------|---------------|
| **Tokio** | `runtime-tokio` (default) | Native (std) | Multi-threaded |
| **smol** | `runtime-smol` | Native (std) | Multi-threaded |
| **async-std** | `runtime-async-std` | Native (std) | Multi-threaded (⚠️ deprecated) |
| **WASM** | `runtime-wasm` | Browser (no_std) | Single-threaded |
| **Embassy** | `runtime-embassy` | Embedded (no_std) | Single-threaded |

⚠️ **Note:** async-std is unmaintained (RUSTSEC-2025-0052). Use Tokio or smol for new projects.

## Features

### Timer Trait

The `Timer` trait provides runtime-agnostic time operations:

```rust
pub trait Timer: Clone + Send + Sync + Debug + 'static {
    type Sleep: Future<Output = ()>;
    type Instant: Copy + Debug + Ord + Send + Sync + ...;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep;
    fn now(&self) -> Self::Instant;
}
```

**Key Benefits:**
- Zero-cost abstraction (compiles to direct runtime calls)
- Type-safe instant handling
- No runtime overhead

### Runtime Implementations

Each runtime has a custom `Timer` implementation optimized for its execution model:

#### TokioTimer (Multi-threaded)
```rust
use fluxion_runtime::impls::tokio::TokioTimer;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let timer = TokioTimer;
    let start = timer.now();
    timer.sleep_future(Duration::from_millis(100)).await;
    let elapsed = timer.now() - start;
    println!("Elapsed: {:?}", elapsed);
}
```

#### SmolTimer (Multi-threaded)
```rust
use fluxion_runtime::impls::smol::SmolTimer;
use std::time::Duration;

fn main() {
    smol::block_on(async {
        let timer = SmolTimer;
        timer.sleep_future(Duration::from_millis(100)).await;
    });
}
```

#### WasmTimer (Browser)
```rust
use fluxion_runtime::impls::wasm::{WasmTimer, WasmInstant};
use std::time::Duration;

// WASM target only
#[cfg(target_arch = "wasm32")]
async fn example() {
    let timer = WasmTimer;
    let start = timer.now();
    timer.sleep_future(Duration::from_millis(100)).await;
    let elapsed = timer.now() - start; // WasmInstant with millisecond precision
}
```

#### EmbassyTimerImpl (Embedded)
```rust
use fluxion_runtime::impls::embassy::EmbassyTimerImpl;
use core::time::Duration;

// Embassy executor context
async fn sensor_task() {
    let timer = EmbassyTimerImpl;
    loop {
        timer.sleep_future(Duration::from_secs(1)).await;
        // Read sensor...
    }
}
```

## Usage with Fluxion Time Operators

The Timer abstraction is primarily used by `fluxion-stream-time` operators (debounce, throttle, delay, sample, timeout). Users typically don't interact with timers directly when using convenience methods:

```rust
use fluxion_stream_time::prelude::*;
use std::time::Duration;

// Convenience API (recommended) - timer chosen automatically
stream.debounce(Duration::from_millis(500));

// Explicit API - for custom timer implementations
stream.debounce_with_timer(Duration::from_millis(500), custom_timer);
```

## Feature Flags

```toml
[dependencies]
fluxion-runtime = "0.8.0"

# Choose your runtime:
# Default: Tokio (no configuration needed)

# Alternative runtimes:
fluxion-runtime = { version = "0.8.0", default-features = false, features = ["runtime-smol"] }
fluxion-runtime = { version = "0.8.0", default-features = false, features = ["runtime-wasm"] }
fluxion-runtime = { version = "0.8.0", default-features = false, features = ["runtime-embassy"] }
```

### Feature Combinations

- **`std`** - Enable standard library support (default)
- **`alloc`** - Enable allocator support for no_std
- **`runtime-tokio`** - Tokio runtime support (includes `std`, `parking_lot`)
- **`runtime-smol`** - smol runtime support (includes `std`, `parking_lot`, `async-io`)
- **`runtime-async-std`** - async-std runtime support (includes `std`, `parking_lot`, `async-io`)
- **`runtime-wasm`** - WASM runtime support (includes `parking_lot`, `gloo-timers`, `js-sys`)
- **`runtime-embassy`** - Embassy runtime support (includes `embassy-time`)

## Architecture

### Multi-threaded Runtimes (Tokio, smol, async-std)

- Use `std::time::Instant` for timestamps
- Thread-safe with `Arc<Mutex>` synchronization
- Spawning support via `FluxionTask` trait

### Single-threaded Runtimes (WASM, Embassy)

- Custom instant types (`WasmInstant`, `EmbassyInstant`)
- No thread bounds required
- Embassy: Static task allocation, no dynamic spawning

## Implementation Details

### Timer Pattern

All time-based operators use a consistent pattern:

```rust
pub struct Debounce<S, T, TM: Timer> {
    source: S,
    duration: Duration,
    timer: TM,
    pending: Option<T>,
    #[pin]
    sleep: Option<TM::Sleep>,
}
```

This pattern enables:
- Zero-cost abstraction (monomorphization)
- Optimal memory layout
- Runtime-specific optimizations

### Instant Types

Each runtime has a custom `Instant` type implementing required traits:

- `Copy + Debug + Ord` - For comparison and debugging
- `Add<Duration>` / `Sub<Duration>` - For time arithmetic
- `Sub<Self, Output = Duration>` - For elapsed time calculation

## Examples

See the complete examples in the workspace:

- **[wasm-dashboard](../examples/wasm-dashboard/)** - WASM timer in browser
- **[embassy-sensors](../examples/embassy-sensors/)** - Embassy timer on ARM Cortex-M4F
- **[stream-aggregation](../examples/stream-aggregation/)** - Tokio timer (default)

## Migration from Pre-0.8.0

If upgrading from versions before the dual trait bound system:

**Before (Tokio-only):**
```rust
use tokio::time::{sleep, Duration};
```

**After (Runtime-agnostic):**
```rust
use fluxion_stream_time::prelude::*;
use std::time::Duration;

// Timer automatically selected based on runtime feature
stream.debounce(Duration::from_millis(500));
```

No code changes needed - the timer abstraction is transparent!

## Advanced: Custom Timer Implementation

To add support for a new runtime, implement the `Timer` trait:

```rust
use fluxion_runtime::timer::Timer;
use core::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct MyCustomTimer;

impl Timer for MyCustomTimer {
    type Sleep = impl Future<Output = ()>;
    type Instant = MyInstant;

    fn sleep_future(&self, duration: Duration) -> Self::Sleep {
        // Your runtime's sleep implementation
    }

    fn now(&self) -> Self::Instant {
        // Your runtime's instant implementation
    }
}
```

## Testing

The crate includes comprehensive tests for all runtime implementations:

```bash
# Test Tokio runtime (default)
cargo test

# Test smol runtime
cargo test --no-default-features --features runtime-smol

# Test WASM runtime
wasm-pack test --node

# Test Embassy runtime (compilation check)
cargo check --target thumbv7em-none-eabihf --no-default-features --features runtime-embassy
```

## Performance

The `Timer` abstraction has **zero runtime overhead**:

- Monomorphization eliminates virtual dispatch
- Compiler optimizes timer calls to direct runtime functions
- No allocations on the hot path
- Optimal memory layout with `#[pin]` projections

## Documentation

- [Main Project README](../README.md)
- [Time Operators Documentation](../fluxion-stream-time/README.md)
- [Embassy Example](../examples/embassy-sensors/README.md)
- [WASM Example](../examples/wasm-dashboard/README.md)

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.
