# Migration Findings - First Operator Attempt

**Date:** December 27, 2025
**Operator:** delay.rs
**Approach:** Direct interface change (no parallel traits)

## Changes Made

Changed `DelayExt` from:
```rust
pub trait DelayExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>
where T: Send, TM: Timer
```

To:
```rust
pub trait DelayExt<T>: Stream<Item = StreamItem<T>>
where T: Fluxion
```

## Compilation Results

**23 errors, 1 warning**

### Critical Issues

#### 1. Timestamp Type Mismatch (BLOCKER)
```
error[E0271]: type mismatch resolving `<TokioTimer as Timer>::Instant == <T as HasTimestamp>::Timestamp`
```

**Problem:** Feature-gated timer selection tries to use `TokioTimer` which has `Instant = std::time::Instant`, but `T::Timestamp` is generic and could be anything.

**Why This Happens:** We're trying to call:
```rust
self.delay_with_timer(duration, crate::TokioTimer)
```

But `delay_with_timer` requires:
```rust
TM: Timer<Instant = T::Timestamp>
```

And `T::Timestamp` is completely generic - we don't know what it is!

**This is the core problem:** We can't auto-select a timer without knowing what timestamp type `T` uses.

#### 2. Conflicting Traits
The old `DelayWithDefaultTimerExt` trait still exists and conflicts with the new approach.

#### 3. Internal Implementation Issues
- `DelayStream` struct needs `S: Stream` bound
- Type parameter juggling in the future/stream implementation
- `DelayFuture` expects wrong types

### Root Cause Analysis

**The fundamental issue:** Feature-gated timer selection doesn't work with generic `T: Fluxion` because:

1. `T: Fluxion` doesn't tell us what `T::Timestamp` is
2. Each timer has a specific `Instant` type (e.g., `std::time::Instant` for Tokio)
3. We can't match them without knowing the concrete timestamp type

**Example:**
```rust
// T could be:
TokioTimestamped<i32, TokioTimer>     // T::Timestamp = std::time::Instant
SmolTimestamped<i32, SmolTimer>       // T::Timestamp = smol::time::Instant
CustomTimestamped<i32>                // T::Timestamp = CustomInstant

// Feature-gated code tries:
self.delay_with_timer(duration, TokioTimer)
// But TokioTimer::Instant = std::time::Instant
// And T::Timestamp could be anything!
```

## Possible Solutions

### Option A: Constrain T::Timestamp in Feature Gates
```rust
#[cfg(all(feature = "runtime-tokio", not(target_arch = "wasm32")))]
impl<S, T> DelayExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion<Timestamp = std::time::Instant>,  // Constraint timestamp type!
    // ...
```

**Problem:** This creates multiple incompatible implementations. `T: Fluxion<Timestamp = std::time::Instant>` and `T: Fluxion<Timestamp = smol::time::Instant>` can't coexist.

### Option B: Runtime Must Determine Timestamp Type
The timestamp type MUST be tied to the runtime feature. We need a way to say:
- "When `runtime-tokio` is enabled, all timestamps are `std::time::Instant`"
- "When `runtime-smol` is enabled, all timestamps are `smol::time::Instant`"

This means `InstantTimestamped<T, TM>` or equivalent MUST be used - we can't have arbitrary timestamp types with feature-gated timers.

### Option C: Associated Type on Timer
```rust
#[cfg(feature = "runtime-tokio")]
type DefaultTimer = TokioTimer;
type DefaultTimestamp = std::time::Instant;

impl<S, T> DelayExt<T> for S
where
    T: Fluxion<Timestamp = DefaultTimestamp>,
```

This ties timestamp type to the runtime feature.

### Option D: Keep Timer Parameter (Current Approach)
Users must pass timer explicitly - no feature-gated auto-selection. This means:
```rust
stream.delay_with_timer(duration, timer)  // User provides timer
```

Not the clean API we wanted:
```rust
stream.delay(duration)  // This can't work generically
```

## Recommendation

**We need to constrain T to use runtime-specific timestamp types.**

The cleanest approach:
1. Define a type alias per runtime: `type TokioTimestamped<T> = InstantTimestamped<T, TokioTimer>`
2. Make operators generic over `T: Fluxion` BUT
3. Feature-gate the convenient `delay()` method to work only with the runtime-specific timestamp type
4. Keep `delay_with_timer<TM>()` fully generic for custom timers

This means:
- ✅ Operators work with any `T: Fluxion`
- ✅ Feature-gated convenience with `TokioTimestamped`/`SmolTimestamped`
- ✅ `InstantTimestamped` still used but not in trait signatures
- ✅ Users can provide custom timers for custom timestamp types

## Next Steps

1. Revert the direct change to `delay.rs`
2. Create parallel trait `DelayExt` that properly handles this
3. Test with runtime-specific implementations
4. Document the timestamp type constraint requirement
