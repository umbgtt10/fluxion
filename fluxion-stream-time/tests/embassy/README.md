# Embassy Timer Tests

## Overview

Embassy timer tests require a **global time driver** implementation. Unlike other runtimes (Tokio, smol, async-std) which provide complete async runtimes, Embassy is designed for embedded systems where the timer driver must be hardware-specific or software-emulated.

## Embassy Time Driver Architecture

Embassy uses a **link-time driver selection** mechanism:

```rust
// Embassy internally defines:
extern "Rust" {
    fn _embassy_time_now() -> u64;
    fn _embassy_time_schedule_wake(at: u64, waker: &core::task::Waker);
}

// Driver crates provide:
#[no_mangle]
fn _embassy_time_now() -> u64 {
    // Hardware timer implementation
}

#[no_mangle]
fn _embassy_time_schedule_wake(at: u64, waker: &core::task::Waker) {
    // Hardware alarm implementation
}
```

The linker resolves these at build time. If no driver is present (our case), linking fails.

## Testing Options

### 1. **Compilation Tests** (Current Approach) ✅
- Tests validate that types compile correctly
- No runtime execution required
- Marked with `#[ignore]` attribute
- **Status**: Implemented

### 2. **Mock Driver Tests** (Possible but Complex)
```toml
[dev-dependencies]
embassy-time = { version = "0.3", features = ["mock-driver"] }
embassy-executor = { version = "0.3", features = ["nightly", "arch-std"] }
```

Requirements:
- Requires `embassy-executor` with std support
- Needs nightly Rust for async tests
- More complex setup for minimal benefit

### 3. **Generic Queue Tests** (Software Timer)
```toml
[dev-dependencies]
embassy-time = { version = "0.3", features = ["generic-queue-32"] }
```

Still requires executor and more setup.

### 4. **Hardware-in-Loop** (Real Embedded Testing)
Users test in their actual embedded projects with real hardware drivers.

## Current Test Structure

### `embassy_instant_tests.rs`
- **Compilation tests** for `EmbassyInstant` wrapper
- **Type safety** validation
- **Duration conversion** correctness (implicit through types)
- **Arithmetic operations** (Add, Sub with `core::time::Duration`)
- **Ordering** (PartialOrd, Ord traits)

All tests are marked `#[ignore]` because they require an embedded environment to run.

## Rationale for Current Approach

1. **Our wrapper is simple**: Just type conversion, no complex logic
2. **Other runtimes tested**: Tokio/smol/async-std prove operator logic works
3. **Compilation verified**: CI checks Embassy timer compiles
4. **Minimal testing overhead**: No need for complex mock setup
5. **Real validation**: Users will test in actual embedded projects

## Running Tests in Embedded Projects

Users should copy these patterns to test in their embedded projects:

```rust
#[embassy_executor::test]
async fn test_fluxion_debounce_with_embassy() {
    use fluxion_stream_time::{EmbassyTimerImpl, EmbassyTimestamped, DebounceExt};
    use core::time::Duration;

    let timer = EmbassyTimerImpl;
    let (tx, rx) = /* your channel */;

    let debounced = rx.debounce(Duration::from_millis(100));

    // Test with actual Embassy executor
}
```

## Future Enhancements

If more thorough testing becomes necessary:

1. Add `mock-driver` feature to dev-dependencies
2. Set up `embassy-executor` with std/nightly
3. Create async test harness
4. Test actual timer delays (not just type compatibility)

For now, the compilation tests + user validation in real projects is sufficient.

## References

- [Embassy Time Driver Documentation](https://docs.embassy.dev/embassy-time-driver/)
- [Embassy Time Crate](https://docs.embassy.dev/embassy-time/)
- [Embassy Executor](https://docs.embassy.dev/embassy-executor/)
