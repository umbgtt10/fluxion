# Runtime Abstraction Status & no_std Roadmap

**Last Updated:** December 21, 2025

## Executive Summary

✅ **Runtime abstraction is COMPLETE** via the `FluxionTask` trait.

**Current State:**
- ✅ Multi-runtime support: Tokio, smol, async-std, WASM (4 runtimes)
- ✅ 100% operator compatibility on WASM (27/27 operators)
- ✅ 1,480 tests passing across all runtimes
- ✅ Zero breaking API changes
- ✅ Production-ready and deployed

**Future Work:**
- 📋 no_std support (7.5 days if requested)

---

## 📋 Open Points: no_std Support

**Scope:** Support no_std environments (embedded, kernel modules, constrained WASM)

**Strategy:** Design poll-based alternatives instead of degrading spawn-based operators

### Philosophy

**Instead of:** Making spawn-based operators work without spawn (compromises, degraded semantics)

**Approach:** Design new operators naturally suited to poll-based execution (honest names, clear semantics)

### Current Compatibility Assessment

**Compatible Operators (24/27):**
- All non-spawning operators
- Uses `alloc` for heap allocations
- Pure Stream transformations

**Spawn-Based Operators (3/27):**
- `share()` - hot multi-subscriber broadcast (needs background task)
- `partition()` - concurrent dual-branch routing (optimized with spawn)
- `subscribe_latest()` - skip intermediate values during slow processing (needs concurrency)

#### Proposed Solutions for no_std

**1. `share()` → New `publish()` Operator**

Instead of degrading `share()`, introduce new operator with different semantics:

```rust
// Lazy shared execution - no spawn needed
pub fn publish(self) -> Published<T, S>
```

**Semantics:**
- Multiple subscribers poll the same source (lazy, not hot)
- First subscriber to poll triggers upstream fetch
- All subscribers receive the same cached value
- Works everywhere (no background task)

**Honest difference:** "publish" (lazy) vs "share" (hot) - different names for different semantics

**Alternative:** Document that `FluxionSubject` already provides hot multi-subscriber pattern without operators

**2. `partition()` → Dual Implementation**

Provide both high-performance and compatible versions:

```rust
// High-performance (std with spawn)
#[cfg(feature = "std")]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F>

// Compatible (no_std poll-based)
#[cfg(not(feature = "std"))]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F>
```

**Implementation:**
- std: Current spawn-based (concurrent branch progress)
- no_std: Poll-based state machine (sequential branch progress)

**Key:** Same name, same semantics, different performance characteristics clearly documented

**3. `subscribe_latest()` → Already Solved**

The use case ("I'm slow, skip intermediate values") is covered by existing operators that work on all runtimes:

- ✅ `throttle()` - rate-limit processing (works everywhere with appropriate timer)
- ✅ `sample()` - sample at intervals (works everywhere with appropriate timer)
- ✅ `debounce()` - skip rapid-fire updates (works everywhere with appropriate timer)

**Example:**
```rust
// Instead of subscribe_latest
stream.throttle(Duration::from_millis(100))
      .subscribe(slow_processor)
```

**Result:** No new operator needed, existing time operators solve the use case

#### Expected Outcome

**With this approach:**
- 24 operators: ✅ Work out-of-box everywhere
- 1 operator (partition): ✅ Dual implementation, works everywhere with documented trade-off
- 1 operator (subscribe_latest): ✅ Use cases covered by throttle/sample/debounce
- 1 operator (share): ✅ Either new `publish()` operator OR document `FluxionSubject` usage
- Optional: ✅ New `publish()` for lazy multi-subscriber pattern

**= 27/27 operators available on all runtimes (with appropriate alternatives)**

---

## 🗺️ Implementation Roadmap

### Phased Approach

Implementation follows dependency order: `fluxion-core` → `fluxion-stream` → `fluxion-stream-time`

### Phase 1: Core Infrastructure (2 days)

**Goal:** Enable no_std compilation for fluxion-core and fluxion-stream

**Changes:**

1. **Add conditional no_std attributes:**
   ```rust
   // fluxion-core/src/lib.rs
   #![cfg_attr(not(feature = "std"), no_std)]

   #[cfg(not(feature = "std"))]
   extern crate alloc;
   ```

2. **Feature flags:**
   ```toml
   [features]
   default = ["std"]
   std = ["futures/std", "futures-util/std"]
   alloc = []

   # Runtime features require std
   runtime-tokio = ["std", "dep:tokio"]
   runtime-smol = ["std", "dep:smol"]
   runtime-async-std = ["std", "dep:async-std"]
   runtime-wasm = ["std"]
   ```

3. **Feature-gate spawn-based operators:**
   ```rust
   #[cfg(feature = "std")]
   pub fn share(self) -> FluxionShared<T, S> { /* ... */ }

   #[cfg(feature = "std")]
   pub fn subscribe_latest(self) -> SubscribeLatest<T, S> { /* ... */ }
   ```

4. **Use core::error::Error:**
   - Requires Rust 1.81+ (stable since August 2024)
   - Already widely available

**Testing:**
```powershell
# Verify no_std compilation
cargo build --target thumbv7em-none-eabihf --no-default-features --features alloc
```

**Effort:** 2 days
- Conditional compilation setup (1 day)
- Testing and validation (1 day)

**Deliverable:** 24/27 operators compile and work on no_std with alloc

---

### Phase 2: Poll-Based partition() (3 days)

**Goal:** Provide no_std-compatible partition() with identical API

**Implementation:**

```rust
// Same function signature, different implementations
#[cfg(feature = "std")]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F> {
    // Current spawn-based implementation (concurrent)
}

#[cfg(not(feature = "std"))]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F> {
    // New poll-based state machine (sequential)
}
```

**State Machine Design:**
- Single-threaded sequential branch progress
- Same semantics as spawn version (true/false routing)
- Documented performance difference (sequential vs concurrent)

**Effort:** 3 days
- State machine implementation (2 days)
- Testing on embedded target (0.5 days)
- Documentation (0.5 days)

**Deliverable:** partition() works on no_std with documented trade-offs (25/27 operators)

---

### Phase 3: Time Operators (2.5 days)

**Goal:** Enable fluxion-stream-time on no_std with embassy-time

**Changes:**

1. **Add conditional compilation:**
   ```rust
   // fluxion-stream-time/src/lib.rs
   #![cfg_attr(not(feature = "std"), no_std)]

   #[cfg(not(feature = "std"))]
   extern crate alloc;
   ```

2. **Embassy timer implementation:**
   ```rust
   // runtimes/embassy_impl.rs
   #[cfg(feature = "timer-embassy")]
   impl Timer for EmbassyTimerImpl {
       type Sleep = embassy_time::Timer;
       type Instant = embassy_time::Instant;

       fn sleep(duration: Duration) -> Self::Sleep {
           embassy_time::Timer::after(embassy_time::Duration::from_micros(
               duration.as_micros() as u64
           ))
       }

       fn now() -> Self::Instant {
           embassy_time::Instant::now()
       }
   }
   ```

3. **Feature flags:**
   ```toml
   [features]
   default = ["std", "timer-tokio"]
   std = []
   alloc = []
   timer-tokio = ["std", "dep:tokio"]
   timer-smol = ["std", "dep:smol"]
   timer-async-std = ["std", "dep:async-std"]
   timer-embassy = ["alloc", "dep:embassy-time"]  # no_std
   ```

**Why This Works:**
- Timer trait already uses core::time::Duration
- Zero-cost abstraction validated by 4 existing runtime impls
- All operator logic is pure (no std dependencies)
- Operators use Box::pin() (requires alloc, which is fine)

**Effort:** 2.5 days
- Embassy implementation (1 day)
- Feature flags and conditional compilation (0.5 days)
- Testing and validation (1 day)

**Deliverable:** All 5 time operators work on embassy/no_std

---

### Phase 4: Optional Enhancements (+3 days)

**Goal:** Additional operators and improvements based on user feedback

**Option A: publish() Operator** (+3 days)
- Lazy multi-subscriber pattern (alternative to share())
- No background task required
- Works on all runtimes including no_std

**Option B: Additional Timer Implementations** (+1 day each)
- RTIC timer support
- Custom embedded timer examples
- Mock timer for deterministic testing

**Option C: Documentation Improvements** (+0.5 days)
- Performance characteristics per operator
- Embedded usage examples
- Migration guide for no_std users

---

## 📊 Effort Summary

| Phase | Duration | Status | Deliverable |
|-------|----------|--------|-------------|
| **Phase 0** | 0.5 days | ✅ **COMPLETE** | std→core/alloc imports (risk-free) |
| **Phase 1** | 2 days | 📋 Pending | Core infrastructure (24/27 operators) |
| **Phase 2** | 3 days | 📋 Pending | Poll-based partition() (25/27 operators) |
| **Phase 3** | 2.5 days | 📋 Pending | Time operators with embassy (all operators) |
| **Phase 4** | +3 days | 📋 Optional | Optional: publish() operator |
| **Total** | **7.5-10.5 days** | ✅ 0.5 / 📋 7-10 | Full no_std support |

**Completed:** Phase 0 (0.5 days) ✅

**Remaining Critical Path:** Phase 1 → Phase 2 → Phase 3 (7.5 days)

**Optional:** Phase 4 based on user demand (+3 days)

---

## 🎯 Success Criteria

- ✅ Compiles with `--no-default-features --features alloc`
- ✅ 24/27 operators work out-of-box on no_std
- ✅ partition() works via poll-based implementation (25/27)
- ✅ Time operators work with embassy-time (all operators with timer)
- ✅ Clear compile errors for std-only operators when std disabled
- ✅ Tests pass on embedded target (thumbv7em-none-eabihf)
- ✅ Documentation explains no_std usage and limitations
- ✅ Zero breaking changes for existing std users

---

## 🔧 Technical Requirements

**Rust Version:**
- Minimum: Rust 1.81+ (for core::error::Error)
- Released: August 2024 (widely available)

**Embedded RequiremenDeliverable |
|-------|----------|-------------|
| **Phase 1** | 2 days | Core infrastructure (24/27 operators) |
| **Phase 2** | 3 days | Poll-based partition() (25/27 operators) |
| **Phase 3** | 2.5 days | Time operators with embassy (all operators) |
| **Phase 4** | +3 days | Optional: publish() operator |
| **Total** | **7.5-10.5 days** | Full no_std support |

**-time` (for embedded timer)

---

## ⚖️ Trade-offs Analysis

| Aspect | With no_std | Without no_std |
|--------|-------------|----------------|
| **Embedded Support** | ✅ ARM, RISC-V, etc. | ❌ Requires std |
| **WASM Binary Size** | ✅ Smaller | ⚠️ Larger |
| **Operator Availability** | 25/27 out-of-box<br>27/27 with alternatives | ✅ 27/27 all work |
| **Complexity** | ⚠️ Feature flags | ✅ Simpler |
| **partition() Performance** | ⚠️ Sequential | ✅ Concurrent |
| **share() Availability** | ❌ Requires std<br>✅ FluxionSubject works<br>✅ Optional publish() | ✅ Works everywhere |
| **subscribe_latest()** | ⚠️ Use throttle/sample<br>(better anyway) | ✅ Available |

---

## 💡 Alternative Solutions for Spawn Operators

### subscribe_latest() → Already Solved

Use case: "I'm slow, skip intermediate values"

**Solution:** Existing time operators cover this:
- ✅ `throttle()` - rate-limit processing
- ✅ `sample()` - sample at intervals
- ✅ `debounce()` - skip rapid-fire updates

**Example:**
```rust
// Instead of subscribe_latest (std-only)
stream.throttle(Duration::from_millis(100))
      .subscribe(slow_processor)  // Works on all runtimes
```

### share() → FluxionSubject or publish()

**Option 1:** Document existing solution
- `FluxionSubject` already provides hot multi-subscriber pattern
- Works on all runtimes without operators
- Zero additional code needed

**Option 2:** New publish() operator (lazy alternative)
- Multiple subscribers poll the same source
- First subscriber to poll triggers upstream fetch
- All subscribers receive same cached value
- No background task needed
- Honest naming: "publish" (lazy) vs "share" (hot)

