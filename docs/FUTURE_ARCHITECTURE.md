# Future Architecture: Complete Runtime Abstraction

**Status**: Planning / Design Phase
**Target**: Post-1.0 (or earlier if bandwidth allows)
**Goal**: Unify all runtime-varying primitives under the `Runtime` trait

---

## Vision

Fluxion should abstract **all runtime-varying primitives** through a single `Runtime` trait, allowing each runtime to optimize for its specific constraints while maintaining zero user-facing impact.

### Design Principle

**"Abstract what varies, compile what's static"**

Each runtime (Tokio, smol, Embassy, WASM) has different optimal implementations for:
- Synchronization primitives (Mutex, RwLock)
- Communication primitives (Channels)
- Task management (Spawning)
- Time primitives (Timers, Instants)

The `Runtime` trait lets each implementation choose the best approach, while feature flags ensure everything compiles down statically at **zero runtime cost**.

---

## Current Architecture (0.8.0)

### Abstraction Status

| Primitive | Current State | Location |
|-----------|--------------|----------|
| **Timer** | ✅ Abstracted | `Runtime::Timer` trait |
| **Instant** | ✅ Abstracted | `Runtime::Instant` type |
| **Mutex** | ⚠️ Partial | `Runtime::Mutex<T>` defined but unused |
| **Channel** | ❌ Direct dependency | `async-channel` everywhere |
| **Spawn** | ❌ Feature-gated | `FluxionTask` with `#[cfg]` branches |

### Inconsistencies

The current architecture uses **4 different patterns**:

1. **Runtime Trait Abstraction** (Timer/Instant)
   - Clean, extensible
   - Each runtime provides optimal implementation

2. **Universal Solution** (Channel)
   - `async-channel` works everywhere identically
   - Simple but prevents runtime-specific optimizations

3. **Feature-Gated Type** (FluxionTask)
   - Different `spawn()` per runtime
   - Works but duplicates logic across `#[cfg]` branches

4. **Module-Level Abstraction** (fluxion_mutex)
   - `parking_lot::Mutex` (std) vs `spin::Mutex` (no_std)
   - Separate from Runtime trait

**These inconsistencies work fine but obscure the underlying pattern**: different runtimes need different primitives.

---

## Future Architecture

### Complete Runtime Trait

```rust
pub trait Runtime: 'static {
    // ✅ Time primitives (already implemented)
    type Timer: Timer<Instant = Self::Instant> + Default;
    type Instant: Copy + Ord + Send + Sync + Debug;

    // 🔜 Synchronization primitives
    type Mutex<T: ?Sized>: MutexLike<T>;
    type RwLock<T: ?Sized>: RwLockLike<T>;

    // 🔜 Communication primitives
    type Sender<T>: SenderLike<T>;
    type Receiver<T>: ReceiverLike<T>;
    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>);

    // 🔜 Task management
    fn spawn<F>(future: F) where F: Future<Output = ()> + Send + 'static;
}
```

### Runtime-Specific Optimizations

Each runtime can optimize for its constraints:

#### Tokio (Multi-threaded, std)
```rust
impl Runtime for TokioRuntime {
    type Mutex<T> = Arc<parking_lot::Mutex<T>>;  // Fast userspace mutex
    type Sender<T> = tokio::sync::mpsc::UnboundedSender<T>;
    type Receiver<T> = tokio::sync::mpsc::UnboundedReceiver<T>;

    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        tokio::sync::mpsc::unbounded_channel()
    }

    fn spawn<F>(future: F) {
        tokio::spawn(future);
    }
}
```

#### Embassy (Embedded, no_std)
```rust
impl Runtime for EmbassyRuntime {
    type Mutex<T> = Arc<spin::Mutex<T>>;  // Spin lock for no_std
    type Sender<T> = StaticSender<T, 32>;  // Fixed-size, no heap
    type Receiver<T> = StaticReceiver<T, 32>;

    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        // Static allocation, compile-time size
        embassy_sync::channel::Channel::new()
    }

    fn spawn<F>(future: F) {
        compile_error!("Embassy does not support dynamic spawning. Use #[embassy_executor::task]")
    }
}
```

#### WASM (Single-threaded)
```rust
impl Runtime for WasmRuntime {
    type Mutex<T> = Rc<RefCell<T>>;  // No atomic overhead!
    type Sender<T> = WasmSender<T>;  // JS-backed channel
    type Receiver<T> = WasmReceiver<T>;

    fn channel<T>() -> (Self::Sender<T>, Self::Receiver<T>) {
        // Could use JS MessageChannel under the hood
        wasm_channel()
    }

    fn spawn<F>(future: F) {
        wasm_bindgen_futures::spawn_local(future);
    }
}
```

### Operator Implementation

Operators use `DefaultRuntime` internally (users never see it):

```rust
// fluxion-stream/src/lib.rs
#[cfg(feature = "runtime-tokio")]
type DefaultRuntime = TokioRuntime;

#[cfg(feature = "runtime-embassy")]
type DefaultRuntime = EmbassyRuntime;

// combine_latest implementation
pub fn combine_latest<...>(...) -> CombineLatest<...> {
    // Uses optimal primitives for selected runtime
    let state = DefaultRuntime::Mutex::new(IntermediateState::new());
    let (tx, rx) = DefaultRuntime::channel();
    // ... operator logic
}
```

### User Experience (Unchanged!)

```rust
// User code looks exactly the same:
rx.into_fluxion_stream()
    .map_ordered(|x| x * 2)
    .debounce(Duration::from_millis(100))
    .subscribe(|x| async move { println!("{}", x) })
    .await?;

// Compiler selects runtime via feature flag:
// [features]
// default = ["runtime-tokio"]  ← Zero-cost, compile-time decision
```

---

## Benefits

### 1. Zero-Cost Abstractions
- All runtime selection happens at compile time
- No vtables, no dynamic dispatch, no runtime overhead
- Each runtime gets exactly the primitives it needs

### 2. Runtime-Specific Optimizations
- **Tokio**: Fast userspace mutexes, optimized channels
- **Embassy**: Static allocation, no heap usage
- **WASM**: No atomic operations, JS integration
- **smol**: Lightweight, minimal overhead

### 3. Clear Error Messages
```rust
// Embassy user trying to use partition():
error[E0080]: evaluation of constant value failed
  --> src/partition.rs:42:9
   |
42 |         DefaultRuntime::spawn(future);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |         Embassy does not support dynamic spawning. Use #[embassy_executor::task]
```

### 4. Extensibility
- Users can implement custom `Runtime` for specialized executors
- Custom runtimes for testing, profiling, or domain-specific needs
- Future runtimes plug in seamlessly

### 5. Architectural Consistency
- One pattern for all runtime concerns
- No more `#[cfg]` spaghetti in operator code
- Easier to maintain and reason about

---

## Migration Path

### Phase 1: Remove Dead Code (0.9.0)
- Remove unused `MutexLike` implementations from `fluxion-runtime`
- Document this architecture vision
- No functional changes

### Phase 2: Spawn Abstraction (0.10.0)
- Add `spawn()` method to `Runtime` trait
- Implement for all runtimes (Tokio, smol, async-std, WASM, Embassy)
- Replace `FluxionTask::spawn()` with `DefaultRuntime::spawn()`
- Remove `FluxionTask` type
- Embassy operators that require spawning get clear compile errors

### Phase 3: Mutex Abstraction (0.11.0)
- Wire up existing `Runtime::Mutex<T>` in all operators
- Replace `Arc<Mutex<T>>` with `DefaultRuntime::Mutex::new(T)`
- Remove `fluxion_mutex` module
- WASM can use `Rc<RefCell<T>>` for zero atomic overhead

### Phase 4: Channel Abstraction (1.0.0)
- Add `SenderLike`/`ReceiverLike` traits
- Add `Runtime::channel()` method
- Replace `async-channel` with `DefaultRuntime::channel()`
- Embassy can use static channels
- Tokio can use `tokio::sync::mpsc` for better integration

### Phase 5: Polish (1.1.0+)
- Add `RwLock` abstraction if needed
- Custom error types per runtime
- Advanced testing infrastructure (mock runtime)

---

## Trade-offs

### Why Not Do This Now?

**Pros of Waiting:**
- Current architecture works (990+ tests passing)
- Users want operators, not internal refactoring
- 15 operators still to implement (27/42 complete)
- Can ship 1.0 with current architecture

**Pros of Doing It Soon:**
- Easier to refactor before 1.0 API stability
- Foundation for future optimizations
- Cleaner codebase for contributors
- Embassy limitations become compile errors (clearer)

**Decision**: Defer to when bandwidth allows. Document now, implement when practical.

---

## Technical Considerations

### Mutex: Runtime or Platform Concern?

**Initial confusion**: Is `Mutex` a runtime concern or a std/no_std concern?

**Answer**: **Both**, but Runtime abstraction is more powerful:
- Tokio/smol/async-std: All want `parking_lot::Mutex` (std)
- Embassy: Wants `spin::Mutex` (no_std)
- WASM: Could use `Rc<RefCell<T>>` (single-threaded, no atomics!)

By putting it in `Runtime`, each can optimize. WASM avoiding atomics is a significant win.

### Channel: Universal vs. Runtime-Specific?

**Current**: `async-channel` is universal (works everywhere)

**Problem**: Prevents optimization:
- Embassy could use static channels (no heap)
- Tokio could use `tokio::sync::mpsc` (better integration)
- WASM could use JS MessageChannel (native integration)

**Solution**: Runtime abstraction lets each optimize.

### Embassy Spawning Limitation

**Current**: 3 operators incompatible with Embassy (`share`, `partition`, `subscribe_latest`)

**With Runtime abstraction**:
```rust
fn spawn<F>(...) {
    compile_error!("Embassy does not support dynamic spawning")
}
```

Clear, early compile error instead of runtime confusion.

---

## API Stability Guarantee

**User-facing API will not change.** This is an internal refactor only.

Before:
```rust
rx.map_ordered(|x| x * 2)
```

After:
```rust
rx.map_ordered(|x| x * 2)  // Identical
```

The `Runtime` abstraction is entirely internal to operator implementations.

---

## Performance Impact

**Zero.** All runtime selection is compile-time:

1. Feature flags select `DefaultRuntime` type
2. Compiler monomorphizes operators with concrete types
3. All trait calls are statically dispatched
4. LLVM inlines everything

**Result**: Identical performance to hand-written runtime-specific code.

---

## Testing Strategy

### Current: Per-Runtime Test Suites
- `tests/tokio/`
- `tests/smol/`
- `tests/embassy/`
- etc.

**Continue this pattern** - ensures each runtime works correctly.

### Future: Mock Runtime
```rust
struct MockRuntime;

impl Runtime for MockRuntime {
    type Mutex<T> = MockMutex<T>;  // Tracks lock/unlock for testing
    type Timer = MockTimer;         // Deterministic time control
    fn spawn<F>(...) { /* track spawned tasks */ }
}
```

Enables deterministic testing, debugging, and profiling.

---

## Related Documents

- [ROADMAP.md](../ROADMAP.md) - Feature roadmap and versioning
- [README.md](../README.md) - Project overview
- [fluxion-runtime README](../fluxion-runtime/README.md) - Current runtime implementation

---

## Summary

**The future architecture unifies all runtime-varying primitives under a single `Runtime` trait**, allowing each runtime to optimize for its constraints while maintaining:
- ✅ Zero user-facing changes
- ✅ Zero runtime overhead
- ✅ Maximum flexibility per runtime

**Current state**: Partially implemented (Timer/Instant abstracted)
**Target state**: Fully unified (Mutex, Channel, Spawn also abstracted)
**When**: When bandwidth allows (no rush, current architecture works)

---

*Document Status: Living document, will be updated as architecture evolves*
*Last Updated: January 13, 2026*
