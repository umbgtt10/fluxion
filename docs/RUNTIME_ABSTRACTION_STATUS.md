# Runtime Abstraction & no_std Status

**Last Updated:** December 27, 2025

## Executive Summary

✅ **Runtime abstraction via Timer trait COMPLETE** - 5 timer implementations (Tokio, smol, async-std, WASM, Embassy)
📋 **Task spawning abstraction IN PROGRESS** - Requires spawner trait for full Embassy integration
✅ **no_std with Embassy** - Viable embedded async solution with full operator support
❌ **no_std without Embassy** - Makes no sense; use Iterator trait instead

**Current State:**
- ✅ All 27 operators work in std environments (Tokio/smol/async-std/WASM)
- ✅ 25/27 operators work in no_std + Embassy (subscribe_latest and partition pending)
- ⚠️ 25/27 operators work in no_std without runtime (but this configuration is not recommended)
- ✅ All 5 time operators work across all runtimes via Timer trait abstraction
- ✅ 1,790+ tests passing across all runtimes and configurations
- ✅ Zero breaking API changes
- ✅ CI-protected no_std compilation

**Path Forward:**
- 📋 Implement TaskSpawner trait abstraction (similar to Timer trait pattern)
- 📋 Enable subscribe_latest and partition with Embassy via spawner injection
- 📋 Deprecate/document against no_std without runtime (use Iterator instead)

---

## 📋 Task Spawning Abstraction (Recommended Path)

### TaskSpawner Trait Pattern [5-7 days effort]

**Goal:** Abstract task spawning to support all runtimes including Embassy

**Rationale:**
- ✅ Mirrors successful Timer trait pattern already used for time operators
- ✅ Enables subscribe_latest and partition on Embassy without performance penalty
- ✅ Single implementation per operator (no cfg-gated variants)
- ✅ No "reduced performance" trade-offs
- ✅ Embassy becomes "just another runtime"

**Implementation Pattern:**

```rust
// 1. Define spawner trait abstraction
pub trait TaskSpawner: Clone + Send + Sync + 'static {
    type Handle: Send + 'static;

    fn spawn<F, Fut>(&self, task: F) -> Self::Handle
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static;
}

// 2. Implement for global runtimes
pub struct GlobalTaskSpawner;
impl TaskSpawner for GlobalTaskSpawner {
    fn spawn<F, Fut>(&self, task: F) -> TaskHandle {
        FluxionTask::spawn(task) // Uses Tokio/smol/async-std/WASM
    }
}

// 3. Implement for Embassy
pub struct EmbassyTaskSpawner {
    spawner: embassy_executor::Spawner,
}
impl TaskSpawner for EmbassyTaskSpawner {
    fn spawn<F, Fut>(&self, task: F) -> TaskHandle {
        // Embassy-specific spawning via injected spawner
    }
}

// 4. Generic base trait (like ThrottleExt)
trait SubscribeLatestExt<T, SP: TaskSpawner> {
    fn subscribe_latest_with_spawner(self, handler: F, spawner: SP);
}

// 5. Convenience traits (like ThrottleWithDefaultTimerExt)
#[cfg(any(feature = "runtime-tokio", ...))]
trait SubscribeLatestWithDefaultSpawnerExt<T> {
    fn subscribe_latest(self, handler: F); // No spawner param
}

// 6. Runtime-specific convenience impls
impl<S, T> SubscribeLatestWithDefaultSpawnerExt<T> for S {
    fn subscribe_latest(self, handler: F) {
        self.subscribe_latest_with_spawner(handler, GlobalTaskSpawner)
    }
}
```

**Tasks:**
1. Define TaskSpawner trait (0.5 days)
2. Implement for global runtimes (1 day)
3. Implement for Embassy (1 day)
4. Refactor subscribe_latest to use spawner (1.5 days)
5. Refactor partition to use spawner (1.5 days)
6. Tests across all runtimes (1 day)
7. Documentation (0.5 days)

**Success Criteria:**
- ✅ All 27 operators work with Embassy
- ✅ Same implementation across all runtimes
- ✅ Zero performance penalty
- ✅ API remains clean via convenience traits
- ✅ Tests pass on all 5 runtimes

---

## ❌ Poll-Based Partition (Obsolete Approach)

**This approach is NO LONGER RECOMMENDED.**

The previous plan to implement poll-based state machines for partition() would have:
- ⚠️ Required separate implementations for std vs no_std
- ⚠️ Sequential branch progress in no_std (performance penalty)
- ⚠️ Increased maintenance burden
- ⚠️ Still wouldn't support subscribe_latest

**Why TaskSpawner is Better:**
- ✅ Single implementation for all runtimes
- ✅ Full concurrency on Embassy (no performance penalty)
- ✅ Solves subscribe_latest AND partition together
- ✅ Follows proven Timer trait pattern

---

## 🚫 no_std Without Runtime: Not Recommended

**Position:** Supporting no_std without an async runtime (Embassy) **makes no sense** for a reactive streams library.

**Rationale:**

### What's Available Without Runtime?
- ✅ Synchronous transforms: `map`, `filter`, `scan`, `take`, `skip`
- ✅ Stateful operators: `combine_latest`, `merge`, `distinct_until_changed`
- ❌ Time-based operators: No timer driver
- ❌ Task spawning: No executor
- ❌ Async subscribe: Cannot drive streams to completion

### The Problem
**This is just Iterator with worse ergonomics.**

```rust
// no_std stream without runtime
sensor_readings
    .map(|x| x * 2)
    .filter(|x| x > 100)
    .take(10)
    // ... now what? Can't subscribe, no async

// Rust already has this - use Iterator instead
sensor_readings
    .iter()
    .map(|x| x * 2)
    .filter(|x| x > 100)
    .take(10)
    .collect()
```

### The Value Proposition of Reactive Streams
1. **Time-based operators** (debounce, throttle, delay, sample, timeout)
2. **Async/concurrent event handling** (subscribe, partition, subscribe_latest)
3. **Backpressure** and flow control

Without a runtime, you lose #1 and #2, which are the **primary benefits**.

### Recommendation
**For embedded/no_std users:**
- ✅ **Use Embassy** - Full reactive streams with all 27 operators
- ✅ Modern embedded async/await with hardware timer integration
- ✅ Zero overhead task spawning with compile-time allocation

**For iterator-style operations:**
- ✅ **Use core::iter::Iterator** - Optimized for synchronous pull-based patterns
- ✅ No runtime overhead
- ✅ More idiomatic for non-reactive use cases

---

## 🎯 Supported Runtime Configurations

### ✅ Recommended Configurations

| Configuration | Operators | Use Case |
|--------------|-----------|----------|
| **std + Tokio/smol** | 27/27 | Server applications, desktop apps |
| **std + async-std** | 27/27 | Legacy (deprecated runtime) |
| **WASM + browser** | 27/27 | Web applications |
| **no_std + Embassy** | 27/27* | Embedded async *(with TaskSpawner impl)* |

### ❌ Not Recommended

| Configuration | Operators | Why Avoid |
|--------------|-----------|-----------|
| **no_std without runtime** | 25/27 | Use `Iterator` instead - better ergonomics |

---

## 📋 Optional Enhancements (Low Priority)

### publish() Operator [3 days effort]

**Goal:** Lazy multi-subscriber pattern without background task

**Current Status:**
- share() requires spawning (works with TaskSpawner)
- FluxionSubject provides hot pattern (works everywhere)

**Proposed:**
- publish() for lazy multicast (pull-based shared polling)
- Works without spawning on all runtimes

**Decision:** Wait for user demand - alternatives exist

### 4. Workspace Feature Management

**Current Pattern:**
```toml
# Workspace root
[workspace.dependencies]
fluxion-core = { ..., default-features = false }

# Every dependent crate must add:
fluxion-core = { workspace = true, features = ["std"] }
```

**Impact:**
- 13 files modified for no_std support
- 7/9 workspace crates need explicit std feature
- Manual process prone to errors

**Assessment:**
- ✅ This is standard Rust ecosystem pattern
- ✅ CI tests protect against mistakes
- ❌ No better alternative exists in current Rust

**Decision:**
- Accept as cost of no_std support
- Document pattern in CONTRIBUTING.md
- Maintain CI protection

**Effort:** 0.5 days (documentation only)

**Priority:** Low - this is idiomatic Rust, not a problem to fix

---

## 📊 Effort Summary

### Recommended Path

| Task | Duration | Priority | Deliverable |
|------|----------|----------|-------------|
| **TaskSpawner trait** | 5-7 days | High | Full Embassy support (27/27 operators) |
| **Documentation** | 0.5 days | Medium | Update guides for Embassy usage |

### Optional Enhancements

| Task | Duration | Priority | Benefit |
|------|----------|----------|---------|
| **publish() operator** | 3 days | Low | Lazy multi-subscriber pattern |

---

## 🎯 Decision Framework

### TaskSpawner Implementation

**Implement when:**
- ✅ Embassy users need subscribe_latest or partition
- ✅ Want to claim "all operators work on all runtimes"
- ✅ Ready to commit to ~1 week of implementation effort

**Benefits:**
- ✅ Unlocks full reactive patterns for embedded
- ✅ Zero performance penalty (full concurrency)
- ✅ Single implementation per operator (clean architecture)
- ✅ Mirrors proven Timer trait pattern

**Current Status:** All other operators work without this, so wait for user demand.

---

### publish() Operator

**Implement when:**
- ✅ Users request lazy multicast pattern
- ✅ FluxionSubject too low-level for common use case

**Skip until:**
- User demand materializes
- FluxionSubject proves insufficient

**Current Status:** Low priority - alternatives exist


## ✅ Success Criteria

**Phase 2 (partition):**
- ✅ Compiles with `--no-default-features --features alloc`
- ✅ Same API signature as std version
- TaskSpawner Implementation:**
- ✅ Trait abstraction compiles on all targets
- ✅ subscribe_latest works with Embassy spawner injection
- ✅ partition works with Embassy spawner injection
- ✅ Same API across all runtimes via convenience traits
- ✅ Tests pass on all 5 runtimes (Tokio, smol, async-std, WASM, Embassy)
- ✅ Zero performance penalty vs current implementations
- ✅ 27/27 operators work on Embassy

**Documentation:**
- ✅ Embassy usage examples with spawner injection
- ✅ Clear runtime configuration guide
- ✅ Migration guide from Timer pattern shows precedent
- ✅ CONTRIBUTING.md documents spawner pattern

---

## 📝 Next Steps

### Immediate (Ready to Start)

1. **Documentation update** [0.5 days]
   - Document current operator availability matrix
   - Add Embassy time operator examples
   - Clarify "no_std without runtime" is not recommended

### High Priority (Wait for User Demand)

2. **TaskSpawner implementation** [5-7 days]
   - Wait for users needing subscribe_latest or partition on Embassy
   - Current 25/27 operators cover most use cases
   - Can be added without breaking changes
   - Follows proven Timer trait pattern

### Low Priority (Wait for User Feedback)

3. **publish() operator** [3 days]
   - Wait for user request for lazy multicast
   - FluxionSubject provides alternative
   - Can be added without breaking changes

---

## 🎯 Current Operator Availability

### All Runtimes (Tokio, smol, async-std, WASM)
✅ **27/27 operators** work out-of-the-box

### Embassy (no_std)
- ✅ **25/27 operators** work today
- ⏳ **2/27 operators** pending TaskSpawner:
  - `subscribe_latest` (requires task spawning)
  - `partition` (requires task spawning)
- ✅ All 5 time operators work via Timer trait

### no_std Without Runtime
- ⚠️ **Not recommended** - use `Iterator` instead
- 25/27 operators technically work but offer no advantage over Iterator
- Missing core value: time-based operators and async event handling
