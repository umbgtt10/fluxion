# Fluxion Future Architecture: Runtime Isolation

**Status:** Planned for v0.9.0
**Purpose:** Solve all current limitations through runtime-specific crates

---

## Overview

The workspace restructuring introduces **runtime isolation** - separating implementations into runtime-specific crates. This architectural change solves all three current limitations simultaneously:

1. ✅ `combine_latest`/`with_latest_from` work in Embassy
2. ✅ Perfect type inference in operator chains
3. ✅ `subscribe_latest`/`partition` work in Embassy

**Key Insight:** The root cause of all limitations is **feature-gated implementations sharing a single trait signature**. Runtime-specific crates allow **different trait signatures per runtime**.
**Note:** v0.7.1 successfully validated Embassy runtime on ARM Cortex-M4F in QEMU, demonstrating that current workarounds (MergedStream pattern) are production-ready. The v0.9.0 architecture will eliminate the need for workarounds entirely.
---

## Current Architecture (v0.7.1)

```
fluxion-stream-time/
  └── throttle.rs
      ├── trait ThrottleExt<T, TM> { ... }  ← One trait for all runtimes
      ├── #[cfg(feature = "runtime-tokio")]
      │   impl ThrottleExt for S { ... }
      ├── #[cfg(feature = "runtime-smol")]
      │   impl ThrottleExt for S { ... }
      ├── #[cfg(feature = "runtime-embassy")]
      │   impl ThrottleExt for S { ... }
      └── ... (50+ #[cfg] attributes across codebase)
```

**Problem:** Single trait signature must satisfy **all** runtimes:
```rust
fn throttle(self, d: Duration) -> impl Stream<Item = StreamItem<T>>
//                                               ^^^^^^^^^^^^^^^^^^^^
//                                No + Send: Must work for Embassy too!
```

---

## Future Architecture (v0.9.0)

```
fluxion-runtime/           ← Runtime abstraction layer
  ├── runtime.rs          ← pub trait Runtime
  ├── timer.rs            ← Timer trait
  └── spawner.rs          ← TaskSpawner trait

fluxion-stream-core/      ← Shared implementations (22 operators)
  └── throttle_impl.rs    ← Generic: throttle_impl<S, T, R: Runtime>(...)

fluxion-tokio/            ← Tokio-specific crate
  └── throttle.rs         ← trait ThrottleExt { fn throttle -> impl Stream + Send }

fluxion-embassy/          ← Embassy-specific crate
  └── throttle.rs         ← trait ThrottleExt { fn throttle -> impl Stream }

... (smol, async-std, WASM)
```

**Solution:** **Five separate trait definitions** - each optimized for its runtime.

---

## The Runtime Trait Pattern

### Core Abstraction

```rust
// fluxion-runtime/src/runtime.rs
pub trait Runtime: 'static {
    /// Mutex type (Arc<Mutex> vs Rc<RefCell>)
    type Mutex<T: ?Sized>: MutexLike<T>;

    /// Timer implementation
    type Timer: Timer<Instant = Self::Instant>;

    /// Task spawner
    type Spawner: TaskSpawner;

    /// Instant type
    type Instant: Clone + Ord + ...;
}
```

### Concrete Runtime: Tokio

```rust
// fluxion-tokio/src/runtime.rs
pub struct TokioRuntime;

impl Runtime for TokioRuntime {
    type Mutex<T: ?Sized> = Arc<Mutex<T>>;       // Thread-safe
    type Timer = TokioTimer;
    type Spawner = GlobalTaskSpawner;             // FluxionTask wrapper
    type Instant = std::time::Instant;
}
```

### Concrete Runtime: Embassy

```rust
// fluxion-embassy/src/runtime.rs
pub struct EmbassyRuntime;

impl Runtime for EmbassyRuntime {
    type Mutex<T: ?Sized> = Rc<RefCell<T>>;      // Single-threaded
    type Timer = EmbassyTimer;
    type Spawner = EmbassyTaskSpawner;            // Injected spawner
    type Instant = EmbassyInstant;
}
```

---

## Embassy Spawner Strategy

### The Challenge

Embassy is unique among supported runtimes: the spawner comes from the executor (provided to `main()`) rather than being globally available. This requires a special initialization pattern.

### The Solution: Feature-Gated Initialization

```rust
// fluxion-embassy/src/runtime.rs
#[cfg(feature = "runtime-embassy")]
thread_local! {
    static SPAWNER: RefCell<Option<Spawner>> = RefCell::new(None);
}

#[cfg(feature = "runtime-embassy")]
impl EmbassyRuntime {
    /// Initialize Embassy runtime with spawner from main
    ///
    /// Must be called once before using operators that spawn tasks
    /// (partition, subscribe_latest)
    pub fn init(spawner: Spawner) {
        SPAWNER.with(|s| {
            if s.borrow().is_some() {
                panic!("EmbassyRuntime::init() called twice!");
            }
            *s.borrow_mut() = Some(spawner);
        });
    }

    pub(crate) fn spawner() -> Spawner {
        SPAWNER.with(|s| {
            s.borrow()
                .clone()
                .expect("EmbassyRuntime::init() must be called in main()")
        })
    }
}

pub struct EmbassyTaskSpawner;

impl TaskSpawner for EmbassyTaskSpawner {
    fn spawn<F: Future<Output = ()> + 'static>(future: F) {
        let spawner = EmbassyRuntime::spawner();
        spawner.spawn(future).ok();
    }
}
```

### User Experience

```rust
// Embassy: One-time initialization (feature-gated)
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    EmbassyRuntime::init(spawner);  // ← Single line of setup

    // Now all operators work identically to other runtimes
    stream
        .partition(|x| *x > 10)
        .subscribe_latest(handler)
}

// Tokio/smol/async-std: No initialization needed
#[tokio::main]
async fn main() {
    // Just works - global spawner
    stream
        .partition(|x| *x > 10)
        .subscribe_latest(handler)
}
```

### Design Rationale

**Why This Trade-Off is Optimal:**

| Aspect | Impact |
|--------|--------|
| **Embassy users** | 1 line in `main()` - negligible cost |
| **Other runtime users** | Zero awareness (feature-gated) |
| **API uniformity** | After `init()`, identical operator APIs |
| **Embassy philosophy** | Explicit initialization (idiomatic) |
| **Future-proof** | Can deprecate if Embassy adds global spawning |

**Key Insight:** The complexity is perfectly isolated behind the `#[cfg(feature = "runtime-embassy")]` gate. Non-Embassy users never see `init()` - it doesn't exist in their compilation.

---

## Shared Implementation Pattern

### Generic Core (No cfg gates!)

```rust
// fluxion-stream-core/src/combine_latest_impl.rs
pub fn combine_latest_impl<S, T, R, IS>(
    stream: S,
    others: Vec<IS>,
) -> impl Stream<Item = StreamItem<CombinedState<...>>>
where
    R: Runtime,  // ← Generic over runtime!
    S: Stream<Item = StreamItem<T>>,
    T: Fluxion,
{
    // Uses R::Mutex - automatically Arc or Rc based on runtime
    let state = R::Mutex::new(IntermediateState::new());

    ordered_merge_with_index(streams)
        .filter_map(move |item| {
            let mut guard = state.lock();  // Works for both!
            // ... shared logic
        })
}
```

### Runtime-Specific Wrapper: Tokio

```rust
// fluxion-tokio/src/combine_latest.rs
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized {
    fn combine_latest<IS>(self, others: Vec<IS>, filter: F)
        -> impl Stream<Item = StreamItem<CombinedState<...>>> + Send + Sync;
        //                                                       ^^^^^^^^^^
        //                                       Tokio CAN promise Send + Sync!
}

impl<S, T> CombineLatestExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync,  // ← Tokio-specific bounds
    T: Fluxion,
{
    fn combine_latest<IS>(self, others: Vec<IS>, filter: F)
        -> impl Stream<Item = StreamItem<CombinedState<...>>> + Send + Sync
    {
        fluxion_stream_core::combine_latest_impl::<_, _, TokioRuntime, _>(
            self, others, filter
        )
    }
}
```

### Runtime-Specific Wrapper: Embassy

```rust
// fluxion-embassy/src/combine_latest.rs
pub trait CombineLatestExt<T>: Stream<Item = StreamItem<T>> + Sized {
    fn combine_latest<IS>(self, others: Vec<IS>, filter: F)
        -> impl Stream<Item = StreamItem<CombinedState<...>>>;
        //                                      ^^^^^^^^^^^^^^
        //                             Embassy: No Send needed!
}

impl<S, T> CombineLatestExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,  // ← No Send required
    T: Fluxion,
{
    fn combine_latest<IS>(self, others: Vec<IS>, filter: F)
        -> impl Stream<Item = StreamItem<CombinedState<...>>>
    {
        fluxion_stream_core::combine_latest_impl::<_, _, EmbassyRuntime, _>(
            self, others, filter
        )
    }
}
```

**Result:** Same core logic, different trait signatures per runtime!

---

## How It Solves Each Limitation

### Problem 1: combine_latest in Embassy

**Before (v0.7.1):**
```rust
// ❌ Single trait, must work for both
fn combine_latest<IS>(...)
where
    IS::Stream: Send + Sync + 'static,  // ← Embassy can't satisfy
```

**After (v0.9.0):**
```rust
// ✅ Separate traits per runtime
// fluxion-tokio
fn combine_latest<IS>(...) where IS::Stream: Send + Sync  // Tokio needs this

// fluxion-embassy
fn combine_latest<IS>(...)  // No Send bound - Embassy doesn't need it
```

**Solution:** Different trait = different bounds = works in Embassy!

---

### Problem 2: Type Inference in Chains

**Before (v0.7.1):**
```rust
// Inconsistent return types
fn throttle(...) -> impl Stream                // No Send
fn map_ordered(...) -> impl Stream + Send      // Has Send
// Compiler struggles: "Does throttle return Send?"
```

**After (v0.9.0):**
```rust
// fluxion-tokio: ALL operators return same bounds
fn throttle(...) -> impl Stream + Send + Sync
fn map_ordered(...) -> impl Stream + Send + Sync
fn filter_ordered(...) -> impl Stream + Send + Sync
// ✅ Perfect inference - all types match!

// fluxion-embassy: ALL operators return same bounds
fn throttle(...) -> impl Stream
fn map_ordered(...) -> impl Stream
fn filter_ordered(...) -> impl Stream
// ✅ Perfect inference - all types match!
```

**Solution:** Consistent bounds per runtime = perfect type inference!

---

### Problem 3: subscribe_latest in Embassy

**Before (v0.7.1):**
```rust
// Uses FluxionTask (global spawn) - doesn't work in Embassy
fn subscribe_latest(self, handler: F) {
    FluxionTask::spawn(async move { ... });  // ❌ Embassy needs injected spawner
}
```

**After (v0.9.0):**
```rust
// Shared core implementation
fn subscribe_latest_impl<S, T, R, F>(stream: S, handler: F)
where
    R: Runtime,
    R::Spawner: TaskSpawner,
{
    R::Spawner::spawn(async move {
        // Generic spawning through Runtime trait
        ...
    });
}

// fluxion-tokio (TokioRuntime::Spawner = GlobalTaskSpawner)
fn subscribe_latest(self, handler: F) {
    subscribe_latest_impl::<_, _, TokioRuntime, _>(self, handler)
}

// fluxion-embassy (EmbassyRuntime::Spawner = EmbassyTaskSpawner)
fn subscribe_latest(self, handler: F) {
    subscribe_latest_impl::<_, _, EmbassyRuntime, _>(self, handler)
}
```

**Solution:** Different trait signatures = Embassy can take spawner parameter!

---

## User Experience

### Current (v0.7.1)

```rust
// User must pick runtime via feature flag
[dependencies]
fluxion-rx = { version = "0.7", features = ["runtime-tokio"] }

// Import from unified crate
use fluxion_stream::*;

// Some operators don't work in Embassy
stream.combine_latest(...)  // ❌ Fails in Embassy
```

### Future (v0.9.0)

```rust
// User picks runtime-specific crate
[dependencies]
fluxion-tokio = "0.9"  // OR fluxion-embassy, fluxion-wasm, etc.

// Import from runtime crate
use fluxion_tokio::*;

// ALL operators work!
stream.combine_latest(...)  // ✅ Works everywhere
    .throttle(100ms)
    .map_ordered(|x| x * 2)  // ✅ No type annotations needed
```

**Benefits:**
- ✅ If it compiles, it works (no runtime surprises)
- ✅ Perfect type inference (no annotations)
- ✅ All 27 operators available in all runtimes
- ✅ Zero `#[cfg]` gates in user code

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│  User Application                                   │
│  uses fluxion-tokio::* (or embassy, wasm, etc.)     │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│  Runtime-Specific Crates (Trait Definitions)        │
│  ├─ fluxion-tokio/    (27 trait impls, ~30 lines ea)│
│  ├─ fluxion-embassy/  (27 trait impls, ~30 lines ea)│
│  ├─ fluxion-wasm/     (27 trait impls, ~30 lines ea)│
│  ├─ fluxion-smol/     (27 trait impls, ~30 lines ea)│
│  └─ fluxion-async-std/(27 trait impls, ~30 lines ea)│
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│  Shared Core Implementations (Generic over Runtime) │
│  ├─ fluxion-stream-core/ (22 ops, ~150 lines each)  │
│  ├─ fluxion-time-core/   (5 ops, ~200 lines each)   │
│  └─ fluxion-exec-core/   (2 ops, ~150 lines each)   │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│  Runtime Abstraction Layer                          │
│  └─ fluxion-runtime/                                │
│      ├─ Runtime trait (Mutex, Timer, Spawner)       │
│      ├─ TokioRuntime, EmbassyRuntime, etc.          │
│      └─ Concrete implementations                    │
└─────────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────────┐
│  Foundation                                         │
│  └─ fluxion-core/                                   │
│      ├─ Fluxion trait                               │
│      ├─ Timestamped trait                           │
│      ├─ StreamItem enum                             │
│      └─ Core types                                  │
└─────────────────────────────────────────────────────┘
```

---

## Code Organization Comparison

### Current (v0.7.1)

```
Per operator (e.g., throttle):
├── 1 trait definition with cfg-gated bounds
├── 1 implementation with cfg-gated runtime code
└── Total: ~110 lines with cfg complexity

All 27 operators: ~3,000 lines
```

**Characteristics:**
- Mixed concerns (trait definition + implementation + runtime handling)
- Heavy use of `#[cfg]` attributes throughout
- Single trait signature must satisfy all runtimes (lowest common denominator)
- Runtime-specific logic interleaved with core logic

### Future (v0.9.0)

```
Per operator:
├── 1 shared core implementation (~80 lines, zero cfg gates)
├── 5 runtime-specific trait definitions (~10 lines each = 50 lines)
└── Total: ~130 lines with clear separation

All 27 operators: ~3,500 lines
```

**Characteristics:**
- Clear separation of concerns (core logic separate from trait definitions)
- Zero `#[cfg]` gates in implementations
- Runtime-appropriate trait signatures (no compromise)
- Core logic shared, trait bounds customized per runtime

**Benefits:**
- Modest code increase (~500 lines, +17%) for significant architectural improvement
- Zero cfg gates in operator implementations
- Single point of maintenance for core logic
- Bug fixes propagate to all runtimes automatically
- Runtime-specific trait signatures enable perfect API per runtime

---

## Execution Phases

### Phase 1: Core Layer Separation

**Goal:** Extract trait definitions from implementations

**Deliverables:**
- `fluxion-core/` - Pure traits (Fluxion, Timestamped, StreamItem)
- `fluxion-runtime/` - Runtime trait + concrete implementations
- Updated dependencies

---

### Phase 2: Runtime Abstraction Layer

**Goal:** Define the Runtime trait and implementations

**Deliverables:**
- `Runtime` trait with associated types (Mutex, Timer, Spawner, Instant)
- `TokioRuntime`, `EmbassyRuntime`, `WasmRuntime`, etc.
- `TaskSpawner` trait for Embassy integration

---

### Phase 3: Shared Core Implementations

**Goal:** Create generic operator implementations

**Deliverables:**
- `fluxion-stream-core/` - 22 stream operators generic over `R: Runtime`
- `fluxion-time-core/` - 5 time operators generic over `R: Runtime`
- `fluxion-exec-core/` - 2 execution operators generic over `R: Runtime`
- Comprehensive testing of core implementations

---

### Phase 4: Runtime-Specific Crates

**Goal:** Create thin wrappers for each runtime

**Deliverables:**
- `fluxion-tokio/` - 27 trait definitions + blanket impls
- `fluxion-embassy/` - 27 trait definitions + blanket impls
- `fluxion-wasm/` - 27 trait definitions + blanket impls
- `fluxion-smol/` - 27 trait definitions + blanket impls
- `fluxion-async-std/` - 27 trait definitions + blanket impls

---

### Phase 5: Migration & Validation

**Goal:** Migrate examples, update documentation, comprehensive testing

**Deliverables:**
- Updated examples (4 examples × new imports)
- Migration guide for users
- Updated documentation across all crates
- Performance benchmarks (verify <5% regression)
- Full test suite passing (1,000+ tests)

---

## Success Criteria

After completion, verify:

**Functionality:**
- ✅ All 27 operators work in all 5 runtimes
- ✅ `combine_latest` works in Embassy
- ✅ `with_latest_from` works in Embassy
- ✅ `subscribe_latest` works in Embassy (with spawner parameter)
- ✅ `partition` works in Embassy

**Developer Experience:**
- ✅ Zero type annotations needed in operator chains
- ✅ Perfect IDE support (rust-analyzer)
- ✅ Clear compiler errors (no cfg confusion)
- ✅ Simple imports (`use fluxion_tokio::*`)

**Code Quality:**
- ✅ Zero `#[cfg]` attributes in operator implementations
- ✅ Clear separation: core logic separate from runtime-specific traits
- ✅ Single point of maintenance per operator
- ✅ <5% performance regression

**Testing:**
- ✅ All 1,000+ tests passing
- ✅ Each runtime tested independently
- ✅ Cross-runtime behavior verified consistent

---

## Migration Path for Users (foreseen...)

### Breaking Changes

**Import paths change:**

```rust
// Before (v0.7.1)
use fluxion_stream::*;
use fluxion_stream_time::*;

// After (v0.9.0)
use fluxion_tokio::*;  // Or fluxion_embassy, fluxion_wasm, etc.
```

**Dependency changes:**

```toml
# Before (v0.7.1)
[dependencies]
fluxion-rx = { version = "0.7", features = ["runtime-tokio"] }

# After (v0.9.0)
[dependencies]
fluxion-tokio = "0.9"  # Or fluxion-embassy = "0.9", etc.
```

### Non-Breaking Changes

**Operator APIs remain unchanged:**
```rust
// Same API, just different import path
stream
    .throttle(Duration::from_millis(100))
    .map_ordered(|x| x * 2)
    .filter_ordered(|x| *x > 10)
// Code is identical, just the `use` statement changes
```

### Compatibility Facade (Optional)

**For gradual migration, keep `fluxion-rx` as facade:**

```toml
# fluxion-rx/Cargo.toml (compatibility facade)
[features]
default = ["runtime-tokio"]
runtime-tokio = ["fluxion-tokio"]
runtime-embassy = ["fluxion-embassy"]
# etc.
```

Users can migrate at their own pace over 1-2 release cycles.

---

## Technical Innovations

### 1. Runtime Trait Pattern

**Novel approach:** Associated types for runtime capabilities (Mutex, Timer, Spawner).

**Benefit:** Single generic implementation adapts to runtime context.

### 2. Zero-Overhead Abstraction

**Key:** `R: Runtime` is zero-sized type, monomorphized away at compile time.

**Result:** Same performance as hand-written runtime-specific code.

### 3. Trait-as-Namespace Pattern

**Innovation:** Same trait name (`ThrottleExt`) in different crates = different types.

**Benefit:** Clean imports, no naming conflicts, natural Rust idioms.

### 4. Generic Test Core Pattern

**Problem:** v0.7.1 has ~8,000 lines of duplicated test code across 4 runtimes.

**Solution:** Follow the same pattern as operators - generic test implementations with runtime-specific wrappers.

#### Current Test Duplication (v0.7.1)

```
fluxion-stream-time/tests/
├── tokio/
│   ├── debounce_test.rs      (110 lines)
│   ├── throttle_test.rs      (105 lines)
│   └── ... (5 operators)
├── smol/
│   └── ... (IDENTICAL logic, different attributes)
├── async_std/
│   └── ... (IDENTICAL logic, different attributes)
└── embassy/
    └── ... (IDENTICAL logic, different attributes)

Total: ~2,000 lines × 4 runtimes = 8,000 lines
```

**Only differences:** Test attributes and timer instantiation.

#### Future Test Architecture (v0.9.0)

```
fluxion-test-core/          ← NEW: Shared test implementations
  ├── time_tests.rs         ← Generic test functions
  ├── stream_tests.rs
  └── lib.rs

fluxion-tokio/tests/
  └── time_operators.rs     ← Thin wrapper (10 lines per test)

fluxion-embassy/tests/
  └── time_operators.rs     ← Thin wrapper (12 lines per test)
```

#### Generic Test Implementation

```rust
// fluxion-test-core/src/time_tests.rs
use fluxion_runtime::Runtime;

/// Generic debounce test - works for ANY runtime
pub async fn test_debounce_basic<R>()
where
    R: Runtime,
    R::Timer: Default,
{
    let (tx, rx) = async_channel::unbounded();
    let stream = rx.into_fluxion_stream::<R>();

    let timer = R::Timer::default();
    let debounced = stream.debounce_with_timer(Duration::from_millis(100), timer);

    tx.send(StreamItem::Value(Sequenced::new(1))).await.unwrap();
    tx.send(StreamItem::Value(Sequenced::new(2))).await.unwrap();

    let results = unwrap_stream(debounced).await;
    assert_eq!(results, vec![2]);
}

// Write once, test everywhere!
```

#### Runtime-Specific Wrapper

```rust
// fluxion-tokio/tests/time_operators.rs
use fluxion_test_core::time_tests;
use fluxion_tokio::TokioRuntime;

#[tokio::test]
async fn test_debounce_basic() {
    time_tests::test_debounce_basic::<TokioRuntime>().await;
}

// fluxion-embassy/tests/time_operators.rs
#[embassy_executor::test]
async fn test_debounce_basic() {
    EmbassyRuntime::init(spawner);  // Embassy-specific setup
    time_tests::test_debounce_basic::<EmbassyRuntime>().await;
}
```

#### Benefits

| Metric | v0.7.1 | v0.9.0 | Improvement |
|--------|--------|--------|-------------|
| **Total test code** | 8,000 lines | 750 lines | **90% reduction** |
| **Maintenance burden** | Update 4 copies | Update once | **4x easier** |
| **Consistency guarantee** | Manual | Automatic | **Zero drift risk** |
| **New runtime cost** | 2,000 lines | 50 lines | **97.5% less** |

**Result:** Same benefits as operator implementations - write once, guarantee consistency, easy to extend.

---

## Conclusion

The workspace restructuring is not just a refactoring - it's a **fundamental architecture improvement** that:

1. **Solves all current limitations** (combine_latest, type inference, subscribe_latest)
2. **Reduces technical debt** (zero cfg gates)
3. **Improves maintainability** (single implementation per operator)
4. **Enables future growth** (easy to add new runtimes)
5. **Maintains compatibility** (operator APIs unchanged)
6. **Guarantees API Surface Consistency** (Within a runtime there are no chaining contraints)

The `Runtime` trait pattern enables true runtime isolation while preserving code reuse through generics. This is the path forward for Fluxion to be the premier reactive streams library across all Rust async runtimes.
