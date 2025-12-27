# Time-Bound Operators Interface Consolidation Plan

**Date:** December 27, 2025
**Objective:** Unify the interface of time-bound operators with non-time-bound operators by adopting the standard `S: Stream<Item = StreamItem<T>>, T: Fluxion` pattern.

---

## Executive Summary

Currently, time-bound operators in `fluxion-stream-time` use specialized trait bounds tied to `InstantTimestamped<T, TM>`, while non-time-bound operators in `fluxion-stream` use a generic `T: Fluxion` bound. This creates inconsistency and reduces composability.

**Proposed Solution:** Refactor all time-bound operators to use the same interface pattern as non-time-bound operators, making the API uniform and more flexible.

### Critical Benefits

**1. Complete Operator Chainability**
Time-bound and non-time-bound operators become fully interchangeable - users can freely chain any operators together without type barriers:
```rust
stream
    .filter_ordered(|x| x > 0)           // Non-time-bound
    .debounce(duration)                  // Time-bound - timer auto-selected!
    .map_ordered(|x| x * 2)              // Non-time-bound
    .throttle(duration)                  // Time-bound - timer auto-selected!
    .take_items(10)                       // Non-time-bound
```

**2. InstantTimestamped Disappears from User API**
Users never need to know `InstantTimestamped` exists - they work directly with their own `Fluxion` types. The abstraction is complete:
```rust
// Users just need: T: Fluxion
let stream: Stream<Item = StreamItem<MyData>> = ...;
stream.debounce(duration);  // Just works! Timer auto-selected!
```

---

## Current State Analysis

### Non-Time-Bound Operators (fluxion-stream)
**Example:** `filter_ordered.rs`

```rust
pub trait FilterOrderedExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    fn filter_ordered<F>(self, predicate: F) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    where
        Self: Send + Sync + Unpin + 'static,
        F: FnMut(&T::Inner) -> bool + Send + Sync + 'static;
}

impl<S, T> FilterOrderedExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: Debug + Ord + Send + Sync + Copy + 'static,
{
    // Implementation...
}
```

### Time-Bound Operators (fluxion-stream-time)
**Example:** `debounce.rs`

```rust
pub trait DebounceExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>> + Sized
where
    T: Send,
    TM: Timer,
{
    fn debounce_with_timer(
        self,
        duration: Duration,
        timer: TM,
    ) -> impl Stream<Item = StreamItem<InstantTimestamped<T, TM>>>;
}

impl<S, T, TM> DebounceExt<T, TM> for S
where
    S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>,
    T: Send,
    TM: Timer,
{
    // Implementation...
}
```

**Key Differences:**
1. Time-bound operators explicitly use `InstantTimestamped<T, TM>` in type signatures
2. Time-bound operators require minimal bounds on `T` (just `Send`, sometimes `Clone`)
3. Time-bound operators expose the `TM: Timer` type parameter
4. Non-time-bound operators work with any `T: Fluxion`

---

## Proposed Changes

### New Interface Pattern for Time-Bound Operators

All time-bound operators should adopt this pattern with **feature-gated timer selection**:

```rust
pub trait DebounceExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: /* Timer-compatible requirements */,
{
    /// Debounces the stream by the specified duration.
    ///
    /// Timer is automatically selected based on runtime:
    /// - `feature = "runtime-tokio"` → TokioTimer
    /// - `feature = "runtime-async-std"` → AsyncStdTimer
    /// - `feature = "runtime-wasm"` → WasmTimer
    fn debounce(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync;
}

impl<S, T> DebounceExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
    T::Inner: Clone + Debug + Ord + Send + Sync + Unpin + 'static,
    T::Timestamp: /* Timer-compatible requirements */,
{
    fn debounce(
        self,
        duration: Duration,
    ) -> impl Stream<Item = StreamItem<T>> + Send + Sync
    {
        // Feature-gated timer selection
        #[cfg(feature = "runtime-tokio")]
        let timer = TokioTimer;

        #[cfg(feature = "runtime-async-std")]
        let timer = AsyncStdTimer;

        #[cfg(feature = "runtime-wasm")]
        let timer = WasmTimer;

        // Implementation uses T directly with auto-selected timer
        // ...
    }
}
```

**For Advanced Users (Optional):**
Provide a separate `_with_timer` variant for custom timer implementations:

```rust
/// Advanced: Use a custom timer implementation
fn debounce_with_timer<TM>(
    self,
    duration: Duration,
    timer: TM,
) -> impl Stream<Item = StreamItem<T>> + Send + Sync
where
    TM: Timer<Instant = T::Timestamp>;
```

### Key Design Decisions

1. **Generic Over `T: Fluxion`:** Operators work with any Fluxion type, not just InstantTimestamped
2. **Feature-Gated Timer Selection:** Timer is automatically selected based on runtime features (tokio/async-std/wasm)
3. **Transparent API:** Users call `debounce(duration)` - no timer parameter needed
4. **Timestamp Compatibility:** Internal constraint ensures timer works with `T::Timestamp`
5. **Uniform Return Types:** All operators return `impl Stream<Item = StreamItem<T>>`
6. **Consistent Bounds:** Same trait bound pattern across all operators

---

## Implementation Strategy: Non-Disruptive Phased Approach

### Overview

To avoid a "big bang" implementation and allow evaluation after each phase, we'll use a **dual API approach** where new traits coexist with old ones during development. This allows:
- Incremental **implementation** (not user migration - this is for us as developers)
- Independent evaluation of each operator
- Ability to test and validate each operator before moving to the next
- Easy rollback if issues arise
- No breaking changes until we're ready

### Approach: Parallel Traits with Deprecation Path

Create new trait variants alongside existing ones using a suffix (e.g., `DebounceV2` or `DebounceFluxion`), then gradually deprecate old traits once new ones are proven stable. Users don't need to migrate incrementally - they'll switch when ready.

---

## Implementation Steps

### Phase 1: Prerequisites & Foundation (Low Risk, ~4 hours)

**Deliverable:** Foundation is solid, no user-facing changes yet

1. **Verify Fluxion Trait Coverage**
   - [ ] Confirm `Fluxion` trait includes all necessary bounds
   - [ ] Check that `InstantTimestamped<T, TM>` can implement `Fluxion`
   - [ ] Ensure `Timer::Instant` meets timestamp requirements
   - [ ] Document timer-timestamp compatibility requirements

2. **Define Timer-Timestamp Constraints**
   - [ ] Document required bounds for `T::Timestamp` to work with timers
   - [ ] Create helper trait if needed (e.g., `TimerCompatible`)
   - [ ] Update `Timer` trait documentation

**Evaluation Point:** Verify trait bounds are correct and `InstantTimestamped` works as expected

---

### Phase 2: Single Operator Proof of Concept (Low Risk, ~3 hours)

**Deliverable:** One operator with both old and new API, fully tested

**Choose simplest operator:** `delay.rs` (least complex, good test case)

1. **Add New Trait Alongside Old**
   ```rust
   // Old trait - unchanged, still works
   pub trait DelayExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>
   where T: Send, TM: Timer
   { /* existing */ }

   // New trait - parallel implementation with transparent timer
   pub trait DelayFluxion<T>: Stream<Item = StreamItem<T>>
   where T: Fluxion
   {
       fn delay(self, duration: Duration)
           -> impl Stream<Item = StreamItem<T>>;

       // Optional: Advanced variant for custom timers
       fn delay_with_timer<TM>(self, duration: Duration, timer: TM)
           -> impl Stream<Item = StreamItem<T>>
       where TM: Timer<Instant = T::Timestamp>;
   }
   ```

2. **Implementation Checklist**
   - [ ] Create new trait `DelayFluxion` in same file
   - [ ] Implement `delay()` with feature-gated timer selection
   - [ ] Implement optional `delay_with_timer()` for advanced use
   - [ ] **Decide on Box::pin strategy for this operator**
   - [ ] Document boxing decision and rationale
   - [ ] Add comprehensive tests for new trait
   - [ ] Test with different feature flags (tokio, async-std, wasm)
   - [ ] Add chainability tests (with non-time-bound operators)
   - [ ] Update documentation with both approaches
   - [ ] Keep old trait unchanged

3. **Testing Strategy**
   - [ ] Old API continues to work (existing tests pass)
   - [ ] New API works with `InstantTimestamped`
   - [ ] New API works with custom Fluxion types
   - [ ] New API chains with non-time-bound operators
   - [ ] Feature flag testing:
     - [ ] Test with `runtime-tokio` feature
     - [ ] Test with `runtime-async-std` feature
     - [ ] Test with `runtime-wasm` feature
     - [ ] Verify correct timer is selected for each
   - [ ] Test `delay_with_timer()` advanced variant
   - [ ] Performance comparison (old vs new)

**Evaluation Point:**
- Does the new API work as expected?
- **Box::pin policy established and documented?**
- Feature-gated timer selection working correctlymics issues?
- Performance impact?
- Migration path clear?

**Decision:** Proceed with other operators or iterate on design?

---

### Phase 3: Expand to All Operators (Medium Risk, ~12 hours)

**Deliverable:** All time-bound operators have dual APIs

Apply the same pattern to remaining operators, one at a time:

#### Phase 3a: Operator - throttle.rs (~2.5 hours)
- [ ] Add `ThrottleFluxion` trait alongside `ThrottleExt`
- [ ] Implement `throttle()` with feature-gated timer
- [ ] Ipply established Box::pin policy (likely box - complex stateful operator)
- [ ] Amplement optional `throttle_with_timer()` for advanced use
- [ ] Add tests (existing + new types + chainability + feature flags)
- [ ] Update documentation

**Evaluation Point:** Verify throttle works correctly with auto timer selection

#### Phase 3b: Operator - debounce.rs (~2.5 hours)
- [ ] Add `DebounceFluxion` trait alongside `DebounceExt`
- [ ] Implement `debounce()` with feature-gated timer
- [ ] Ipply established Box::pin policy (likely box - complex stateful operator)
- [ ] Add tests (existing + new types + chainability + feature flags)
- [ ] Update documentation

**Evaluation Point:** Verify debounce works correctly with auto timer selection

#### Phase 3c: Operator - timeout.rs (~2.5 hours)
- [ ] Add `TimeoutFluxion` trait alongside `TimeoutExt`
- [ ] Implement `timeout()` with feature-gated timer
- [ ] Implement optional `timeout_with_timer()` for advanced use
- [ ] Apply established Box::pin policy (likely no box - simple wrapper)
- [ ] Update error handling for generic T
- [ ] Add tests (existing + new types + chainability + feature flags)
- [ ] Update documentation

**Evaluation Point:** Verify timeout and error handling work correctly with auto timer

#### Phase 3d: Operator - sample.rs (~2.5 hours)
- [ ] Add `SampleFluxion` trait alongside `SampleExt`
- [ ] Implement `sample()` with feature-gated timer
- [ ] Implement optional `sample_with_timer()` for advanced use
- [ ] Apply established Box::pin policy (likely box - complex stateful operator)
- [ ] Implement optional `sample_with_timer()` for advanced use
- [ ] Add tests (existing + new types + chainability + feature flags)
- [ ] Update documentation

**Evaluation Point:** Verify sample works correctly with auto timer selection

#### Phase 3e: Cross-Operator Testing (~2 hours)
- [ ] Test chaining multiple new operators together
- [ ] Test mixing old and new API (if needed)
- [ ] Test complex real-world scenarios
- [ ] Performance benchmarking across all operators
Box::pin usage consistent and documented?
- Performance characteristics predictable?
-
**Evaluation Point:**
- All operators working correctly?
- Any patterns that need refinement?
- Ready for user feedback?

---

### Phase 4: User Preview & Feedback (Low Risk, ~2 weeks)

**Deliverable:** Documentation and examples showcasing new API

1. **Documentation & Examples**
   - [ ] Add comprehensive guide comparing old vs new API
   - [ ] Update example projects to use new API (in separate branches)
   - [ ] Create migration guide with real-world examples
   - [ ] Document benefits and trade-offs

2. **Preview Release**
   - [ ] Release as minor version (e.g., 0.X.0)
   - [ ] Both APIs available
   - [ ] Announce new API as "preview" or "experimental"
   - [ ] Gather feedback from early adopters

3. **Iteration**
   - [ ] Address feedback
   - [ ] Refine API based on real usage
   - [ ] Update documentation based on confusion points

**Evaluation Point:**
- User feedback positive?
- Any API ergonomics issues discovered?
- Performance acceptable?
- Ready for deprecation path?

---

### Phase 5: Deprecation Path (Low Risk, ~4 hours)

**Deliverable:** Old API marked deprecated, clear migration path

1. **Add Deprecation Warnings**
   ```rust
   #[deprecated(
       since = "0.X.0",
       note = "Use DelayFluxion trait instead for better composability. \
               See migration guide: https://..."
   )]
   pub trait DelayExt<T, TM>: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>
   { /* ... */ }
   ```

2. **Update All Old Traits**
   - [ ] Add deprecation notices to all old traits
   - [ ] Point to migration guide
   - [ ] Ensure old API still works

3. **Documentation Updates**
   - [ ] Update all primary docs to show new API
   - [ ] Keep old API docs for reference
   - [ ] Clear migration examples

**Evaluation Point:**
- Deprecation warnings clear and helpful?
- Migration guide complete?
- Timeline for removal communicated?

---

### Phase 6: Removal (Breaking Change, ~2 hours)

**Deliverable:** Clean API with only new traits (in major version bump)

1. **Remove Old Traits**
   - [ ] Delete old trait definitions
   - [ ] Remove old trait implementations
   - [ ] Update re-exports

2. **Rename New Traits** (Optional)
   - [ ] Rename `DelayFluxion` → `DelayExt`
   - [ ] Or keep `Fluxion` suffix for clarity

3. **Major Version Release**
   - [ ] Release as major version (e.g., 1.0.0)
   - [ ] Update CHANGELOG with breaking changes
   - [ ] Announce migration complete

**Evaluation Point:** Clean, unified API achieved

---

### Rollback Strategy

At each phase, if issues arise:
- **Phase 1-2:** No user impact, iterate freely
- **Phase 3:** New APIs are additive, can be marked `#[doc(hidden)]` if needed
- **Phase 4:** Keep preview status, continue iteration
- **Phase 5:** Remove deprecation notices, keep both APIs longer
- **Phase 6:** Don't proceed until confidence is high

---

## Affected Files (Non-Disruptive Approach)

### Phase 1-2 (No User Impact)
- `fluxion-core/src/fluxion.rs` - Verify trait bounds (read-only analysis)
- `fluxion-stream-time/src/instant_timestamped.rs` - Verify implementation (read-only analysis)
- `fluxion-stream-time/src/timer.rs` - Document constraints (comments only)

### Phase 3 (Additive Changes Only)
- `src/delay.rs` - Add new `DelayFluxion` trait (parallel to old)
- `src/throttle.rs` - Add new `ThrottleFluxion` trait (parallel to old)
- `src/debounce.rs` - Add new `DebounceFluxion` trait (parallel to old)
- `src/timeout.rs` - Add new `TimeoutFluxion` trait (parallel to old)
- `src/sample.rs` - Add new `SampleFluxion` trait (parallel to old)
- `src/lib.rs` - Re-export new traits (additive)
- `src/prelude.rs` - Add new traits to prelude (additive)

### Phase 4-5 (Documentation Only)
- All operator files - Add deprecation notices
- `README.md` - Update examples to show new API
- `MIGRATION_GUIDE.md` - New file documenting transition
- `CHANGELOG.md` - Document new APIs and deprecations
- `examples/` - Add parallel examples using new API

### Phase 6 (Breaking Changes - Major Version)
- All files from Phase 3 - Remove old traits
- Possibly rename new traits to remove suffix

**Key Point:** Until Phase 6, all changes are backward compatible!

---

## Benefits

### 1. **🎯 Complete Operator Chainability (CRITICAL)**
**Time-bound and non-time-bound operators are fully interchangeable:**
- No type barriers between operator categories
- Chain any operators in any order
- Natural composition without special consideration
- Type inference flows seamlessly through chains

```rust
// Before: This was difficult or impossible
stream_of_instant_timestamped
    .debounce_with_timer(duration, timer)  // Need InstantTimestamped AND timer!
    .filter_ordered(...)  // Type mismatch!

// After: Completely natural - NO timer param, NO InstantTimestamped!
stream
    .filter_ordered(...)
    .debounce(duration)        // Timer auto-selected!
    .map_ordered(...)
    .throttle(duration)        // Timer auto-selected!
```

### 2. **🔒 Complete Abstraction (CRITICAL)**
**`InstantTimestamped` disappears from user-facing API AND timer selection is automatic:**
- Users work directly with their own types
- No need to understand internal timestamp wrappers
- **No need to pass timer objects - fully transparent**
- Time operations just work based on cargo features
- Simpler mental model: "time operators just work"

```rust
// Before: User must know about InstantTimestamped AND pass timer
let timer = TokioTimer;
let stream: Stream<Item = StreamItem<InstantTimestamped<MyData, TokioTimer>>> = ...;
stream.debounce_with_timer(duration, timer);

// After: User just needs their type to be Fluxion
let stream: Stream<Item = StreamItem<MyData>> = ...;
stream.debounce(duration);  // Timer auto-selected! Completely transparent!
```

### 3. **API Consistency**
- Single, uniform interface pattern across all operators
- Easier to learn and remember
- Predictable behavior
- Documentation uses one pattern everywhere

### 4. **Greater Flexibility**
- Operators work with any Fluxion type
- Users can create custom timestamped types
- Enables advanced use cases (multiple timestamp types, custom ordering)
- No forced coupling to specific timestamp implementations

### 5. **Future-Proofing**
- Easier to add new operators following established pattern
- Maintains consistency as library evolves
- Reduces maintenance burden
- Clear extension points for advanced features

---

## Potential Challenges

### 1. **Box::pin Inconsistency Across Operators**

**Issue:** Currently, some time-bound operators use `Box::pin` in their implementations while others don't. This creates:
- Inconsistent performance characteristics
- Unpredictable allocation patterns
- Mixed mental models about when boxing occurs

**Current State:**
```rust
// Some operators (debounce.rs, throttle.rs):
fn debounce_with_timer(...) -> impl Stream<...> {
    Box::pin(DebounceStream { ... })  // Boxing!
}

// Others (delay.rs, timeout.rs):
fn delay_with_timer(...) -> impl Stream<...> {
    DelayStream { ... }  // No boxing
}
```

**Solutions:**
1. **Standardize During Migration**
   - Decide on a consistent policy: always box, never box, or box only when necessary
   - Document the rationale clearly
   - Apply consistently across all new operators

2. **Recommendation: Box Only When Necessary**
   - Complex stateful operators (debounce, throttle, sample) → `Box::pin` (already have internal state)
   - Simple transforming operators (delay, timeout) → No boxing (just wrapping)
   - Add documentation explaining the choice for each

3. **Implementation in New Traits**
   ```rust
   // Simple operators - no boxing needed
   fn delay(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
       DelayStream { ... }
   }

   // Complex stateful operators - box for easier implementation
   fn debounce(self, duration: Duration) -> impl Stream<Item = StreamItem<T>> {
       Box::pin(DebounceStream { ... })
   }
   ```

4. **Benefits of Addressing Now**
   - Consolidation provides perfect opportunity to standardize
   - All operators being touched anyway
   - Can establish clear guidelines for future operators
   - Users get predictable performance characteristics

**Decision Point (Phase 2):** Establish boxing policy during POC implementation

---

### 2. **Timer-Timestamp Compatibility**

**Issue:** Need to ensure `T::Timestamp` is compatible with timer operations

**Solutions:**
- Add constraint: `TM: Timer<Instant = T::Timestamp>`
- Document timer compatibility requirements
- Provide helper utilities for timestamp conversion if needed

---

### 3. **Feature Flag Mutual Exclusion**

**Issue:** Multiple runtime features enabled simultaneously could cause conflicts

**Solutions:**
- Use `compile_error!` macro to enforce mutual exclusion
- Provide clear error messages about which features conflict
- Document in Cargo.toml and README

---

### 4. **Type Inference Complexity**

**Issue:** More generic code might require more type annotations

**Solutions:**
- Provide clear examples showing type inference
- Use `turbofish` syntax in examples where needed
- Consider type aliases for common patterns

---

### 5. **Performance Considerations**

**Issue:** More generic code might impact compile times or runtime performance

**Solutions:**
- Benchmark before and after changes
- Profile generated code for regressions
- Use inline hints where appropriate
- Monitor binary size changes

---

### 6. **Internal Implementation Complexity**

**Issue:** Operator internals need to handle generic `T` instead of concrete `InstantTimestamped`

**Solutions:**
- Use `HasTimestamp` trait for accessing timestamps
- Leverage existing Fluxion trait methods
- RFor End Users: What Changes

### The Good News: Eventually a Breaking Change, But With Deprecation Period

When we eventually complete this consolidation and remove old APIs (Phase 6), users will see:

### Before (Old API - Eventually Deprecated)
```rust
use fluxion_stream_time::{DebounceExt, InstantTimestamped, TokioTimer};

let timer = TokioTimer;
let stream: Stream<Item = StreamItem<InstantTimestamped<i32, TokioTimer>>> = /* ... */;
let debounced = stream.debounce_with_timer(Duration::from_millis(100), timer);
```

### After (New API - Eventually Only API)
```rust
use fluxion_stream_time::DebounceExt;
use fluxion_stream::FilterOrderedExt;  // Can mix freely!

// Work directly with your type - no InstantTimestamped, no timer!
let stream: Stream<Item = StreamItem<MyFluxionType>> = /* ... */;

// Chain freely with any operators - timer auto-selected from features!
let result = stream
    .debounce(Duration::from_millis(100))    // No timer param!
    .filter_ordered(|x| x > 0)               // Non-time-bound operator
    .throttle(Duration::from_millis(50));    // No timer param!
```

**Note:** During Phases 2-5, both APIs coexist. Users can stay on old API until they're ready to switch. This phased approach is for **our implementation process**, not user migration.

- [ ] Remove explicit `InstantTimestamped` type annotations where possible
- [ ] Ensure your types implement `Fluxion` (most already do)
- [ ] Update chained operator calls (simpler now!)
- [ ] Verify timer compatibility with your timestamp types

---

## Timeline Estimate (Non-Disruptive Approach)

- **Phase 1 (Prerequisites):** 4 hours
- **Phase 2 (Single Operator POC):** 3 hours
  - *Evaluation checkpoint* (~30 min)
- **Phase 3 (All Operators):** 12 hours
  - Phase 3a-3d: 2.5 hours each = 10 hours
  - Phase 3e: 2 hours
  - *Evaluation checkpoints* after each operator (~30 min each)
- **Phase 4 (User Preview):** 2 weeks of feedback + 2 hours documentation
  - *Major evaluation checkpoint*
- **Phase 5 (Deprecation):** 4 hours
  - *Evaluation checkpoint* (~30 min)
- **Phase 6 (Removal):** 2 hours

**Total Development Time:** ~25 hours (spread over 3+ weeks with feedback periods)
**Total Elapsed Time:** 3-6 weeks (including feedback and evaluation)

### Key Advantage
Each phase delivers working, testable code with **no disruption to existing users**. Can pause, iterate, or rollback at any checkpoint.

---

## Success Criteria

- [ ] **All time-bound operators use `T: Fluxion` trait bounds**
- [ ] **All operators accept generic `Stream<Item = StreamItem<T>>`**
- [ ] **`InstantTimestamped` removed from all trait signatures**
- [ ] **Time-bound and non-time-bound operators chain seamlessly**
- [ ] **Users can use their own Fluxion types with time operators**
- [ ] Timer type parameter is at method level, not trait level
- [ ] All existing tests pass with updated API
- [ ] New tests demonstrate flexibility with different Fluxion types
- [ ] New tests demonstrate operator chaining (time-bound ↔ non-time-bound)
- [ ] Documentation is complete and consistent
- [ ] Examples compile and run successfully
- [ ] No performance regressions
- [ ] Migration guide is clear and complete
- [ ] API provides complete abstraction over timestamp implementation

---

## Open Questions

1. **Trait Naming Convention**
   - Option A: Suffix approach (`DelayFluxion`, `DebounceFluxion`)
   - Option B: V2 approach (`DelayExtV2`, `DebounceExtV2`)
   - Option C: Module approach (`fluxion::delay::Ext` vs old `time::delay::Ext`)
   - **Recommendation:** Start with suffix, evaluate during Phase 2

2. **Method Naming for Main API**
   - New clean API: `debounce(duration)` (no `_with_timer` suffix)
   - Advanced API: `debounce_with_timer(duration, timer)` for custom timers
   - **Recommendation:** Use clean names without suffix for auto-timer variants

3. **Feature Flag Strategy**
   - Mutually exclusive features? (`runtime-tokio` XOR `runtime-async-std`)
   - Or priority order if multiple enabled?
   - **Recommendation:** Enforce mutual exclusion via build.rs or compile_error!

4. **Default Timer Selection**
   - What happens if no runtime feature enabled?
   - Compile error? Default to std timer?
   - **Recommendation:** Compile error with helpful message

5. **Timer Compatibility Validation**
   - Do we need a helper trait for timer-compatible timestamps?
   - E.g., `trait TimerCompatible: Timestamp + Add<Duration> + Sub<Duration>`
   - **Recommendation:** Evaluate need during Phase 1

6. **Deprecation Timeline**
     Box::pin Policy (CRITICAL - Decide in Phase 2)**
   - Option A: Always box for consistency
   - Option B: Never box for performance
   - Option C: Box only complex stateful operators (debounce, throttle, sample)
   - **Recommendation:** Option C - box only when implementation complexity requires it
   - **Document:** Clear guidelines for when to box

2. **Trait Naming Convention**
   - Option A: Suffix approach (`DelayFluxion`, `DebounceFluxion`)
   - Option B: V2 approach (`DelayExtV2`, `DebounceExtV2`)
   - Option C: Module approach (`fluxion::delay::Ext` vs old `time::delay::Ext`)
   - **Recommendation:** Start with suffix, evaluate during Phase 2

3. **Method Naming for Main API**
   - New clean API: `debounce(duration)` (no `_with_timer` suffix)
   - Advanced API: `debounce_with_timer(duration, timer)` for custom timers
   - **Recommendation:** Use clean names without suffix for auto-timer variants

4. **Feature Flag Strategy**
   - Mutually exclusive features? (`runtime-tokio` XOR `runtime-async-std`)
   - Or priority order if multiple enabled?
   - **Recommendation:** Enforce mutual exclusion via build.rs or compile_error!

5. **Default Timer Selection**
   - What happens if no runtime feature enabled?
   - Compile error? Default to std timer?
   - **Recommendation:** Compile error with helpful message

6. **Timer Compatibility Validation**
   - Do we need a helper trait for timer-compatible timestamps?
   - E.g., `trait TimerCompatible: Timestamp + Add<Duration> + Sub<Duration>`
   - **Recommendation:** Evaluate need during Phase 1

7
