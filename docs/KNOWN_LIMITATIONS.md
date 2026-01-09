# Fluxion Known Limitations & Alternatives

**Last Updated:** January 8, 2026
**Status:** Current limitations in v0.7.0

---

## Overview

Fluxion has two types of limitations:

1. **Interface Inconsistencies (⚠️)** - Can be fixed through runtime abstraction consolidation
2. **Fundamental Limitations (❌)** - Cannot be overcome due to Embassy's execution model

**Summary:**
- ✅ **All operators** work perfectly in Tokio/smol/async-std
- ⚠️ **2 operators** (combine_latest, with_latest_from) need interface fixes for Embassy/WASM
- ❌ **3 operators** (subscribe_latest, partition, share) fundamentally incompatible with Embassy
- ✅ **WASM fully supports** task spawning operators (subscribe_latest, partition, share)

---

## Limitation 1: Interface Inconsistencies (⚠️ - Fixable)

### Problem

`combine_latest` and `with_latest_from` have overly restrictive trait bounds that prevent them from working in Embassy/WASM.

```rust
// ⚠️ Currently fails in Embassy/WASM due to interface design
use fluxion_stream::CombineLatestExt;

stream1.combine_latest(
    vec![stream2, stream3],
    |state| state.values().len() == 3
)
// Error: cannot be sent between threads safely
```

### Root Cause

**This is an interface design issue, not a runtime capability issue.**

The operators require `Send + Sync` bounds on input streams:

```rust
// Current (too restrictive)
fn combine_latest<IS>(self, others: Vec<IS>, ...)
where
    IS::Stream: Send + Sync + 'static,  // ← Unnecessary for single-threaded runtimes
```

While WASM and Embassy CAN work with `Arc<Mutex<T>>` (they're single-threaded, so `Send` is automatic), the **trait signature** prevents compilation.

### Resolution Path

**This WILL be fixed** through runtime abstraction consolidation (v0.9.0):

```rust
// Future (flexible)
fn combine_latest<R: Runtime, IS>(self, others: Vec<IS>, ...)
where
    IS::Stream: 'static,
    // Runtime determines actual bounds needed
```

By abstracting over runtimes, we can relax bounds for single-threaded contexts while maintaining safety.

### Alternative: MergedStream

**Use `MergedStream::seed().merge_with()` pattern:**

```rust
// ✅ Works in all runtimes
use fluxion_stream::{MergedStream, MergeWithExt};

let fused = MergedStream::seed::<EmbassyTimestamped<State>>(State::new())
    .merge_with(stream1, |val1, state| {
        state.update_from_stream1(val1);
        state.clone()
    })
    .merge_with(stream2, |val2, state| {
        state.update_from_stream2(val2);
        state.clone()
    })
    .merge_with(stream3, |val3, state| {
        state.update_from_stream3(val3);
        state.clone()
    });
```

**Why it works:** `MergedStream` only requires `Send` on the state type, not on input streams.

**Trade-offs:**
- ✅ Works in all 5 runtimes
- ✅ Gives explicit control over state updates
- ⚠️ More verbose than `combine_latest`
- ⚠️ Manual state management (vs automatic in combine_latest)

### Alternative: ordered_merge for Simple Cases

**If you don't need combined state:**

```rust
// ✅ Works in all runtimes
use fluxion_stream::OrderedStreamExt;

let merged = stream1.ordered_merge(vec![stream2, stream3]);
// Emits all items from all streams in temporal order
```

**Use when:** You want all items, not just latest combinations.

---

## Limitation 2: Type Inference in Operator Chains

### Problem

Chaining time-bound operators (throttle, debounce, etc.) with non-time-bound operators (map_ordered, filter_ordered) occasionally requires explicit type annotations.

```rust
// ⚠️ May require type annotations
stream
    .debounce(Duration::from_millis(500))
    .map_ordered(|x| x * 2)  // ← Type inference may fail here
```

### Root Cause

**Inconsistent return type guarantees:**
- **Time operators** return `impl Stream<Item = StreamItem<T>>` (no `+ Send` guarantee)
- **Non-time operators** return `impl Stream<Item = StreamItem<T>> + Send + Sync`

Time operators cannot promise `+ Send` because Embassy implementations don't have `Send`. The compiler struggles to infer types when mixing these bounds in complex chains.

### Alternative 1: Explicit Type Annotations

**Add type to closure parameters:**

```rust
// ✅ Compiles reliably
stream
    .debounce(Duration::from_millis(500))
    .map_ordered(|item: TokioTimestamped<i32>| {  // ← Explicit type
        TokioTimestamped::new(item.value * 2, item.timestamp)
    })
```

### Alternative 2: Intermediate Variables

**Break chain into steps:**

```rust
// ✅ Compiles reliably
let debounced = stream.debounce(Duration::from_millis(500));
let processed = debounced.map_ordered(|item| {
    TokioTimestamped::new(item.value * 2, item.timestamp)
});
```

### Alternative 3: Manual Boxing

**Add explicit boxing:**

```rust
// ✅ Compiles reliably
let processed = Box::pin(stream.debounce(Duration::from_millis(500)))
    .map_ordered(|item| item * 2);
```

**When it happens:**
- Mostly when chaining time → non-time operators
- Complex chains with multiple transformations
- Generic functions with complex type parameters

**When it doesn't happen:**
- Simple chains (2-3 operators)
- Non-time → time chains (usually work)
- Same-category chains (time → time, non-time → non-time)

---

## Limitation 3: Task Spawning Operators in Embassy (❌ - Fundamental)

### Problem

`subscribe_latest`, `partition`, and `share` are **fundamentally incompatible** with Embassy's execution model.

```rust
// ❌ Cannot work in Embassy (fundamental limitation)
stream.subscribe_latest(|item| async move {
    process(item).await;
});
```

### Root Cause

**This is a fundamental architectural difference that CANNOT be fixed:**

**Tokio/smol/async-std/WASM execution model:**
```rust
// Can spawn arbitrary futures with captured state
FluxionTask::spawn(|cancel| async move {
    let captured_data = expensive_closure_state;
    // ... use captured_data
});
```

**Embassy execution model:**
```rust
// Tasks MUST be static functions marked at compile time
#[embassy_executor::task]
async fn my_task(params: StaticParams) {
    // Cannot capture arbitrary closure state
}
```

Embassy requires all tasks to be `#[embassy_executor::task]`-annotated static functions. You cannot:
- ❌ Spawn closures with captured state
- ❌ Dynamically create tasks at runtime
- ❌ Pass arbitrary futures to a spawner

This is **by design** for Embassy's no_std embedded targets with static memory allocation.

### Alternative 1: Manual Polling Pattern

**For subscribe_latest equivalent:**

```rust
// ✅ Works in Embassy
let mut stream = source.into_fluxion_stream();

loop {
    if cancel_token.is_cancelled() {
        break;
    }

    match stream.next().await {
        Some(StreamItem::Value(item)) => {
            // Process item
            process(item).await;
        }
        Some(StreamItem::Error(e)) => {
            // Handle error
            log_error(e);
        }
        None => break,  // Stream ended
    }
}
```

### Alternative 2: Manual Partition Pattern

**For partition equivalent:**

```rust
// ✅ Works in Embassy
let mut stream = source.into_fluxion_stream();
let (tx_true, rx_true) = async_channel::unbounded();
let (tx_false, rx_false) = async_channel::unbounded();

spawner.spawn(async move {
    while let Some(item) = stream.next().await {
        match item { Type |
|----------|-------|------|-----------|------|---------|------|
| **Stream Operators (20)** | ✅ | ✅ | ✅ | ✅ | ✅ | - |
| `combine_latest` | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | Interface |
| `with_latest_from` | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | Interface |
| **Time Operators (5)** | ✅ | ✅ | ✅ | ✅ | ✅ | - |
| `subscribe` | ✅ | ✅ | ✅ | ✅ | ✅ | - |
| `subscribe_latest` | ✅ | ✅ | ✅ | ✅ | ❌ | Fundamental |
| `partition` | ✅ | ✅ | ✅ | ✅ | ❌ | Fundamental |
| `share` | ✅ | ✅ | ✅ | ✅ | ❌ | Fundamental |

**Legend:**
- ✅ **Works perfectly** - No workarounds needed
- ⚠️ **Interface issue** - Will be fixed in v0.9.0 through runtime consolidation
- ❌ **Fundamental limitation** - Cannot be fixed; Embassy's execution model incompatible

**Key Insight:** WASM fully supports task spawning operators! Only Embassy has fundamental limitations.d rx_false as separate streams
```

---

## Compatibility Matrix

| Operator | Tokio | smol | async-std | WASM | Embassy |
|----------|-------|------|-----------|------|---------|
| **Stream Operators (20)** | ✅ | ✅ | ✅ | ✅ | ✅ |
| `combine_latest` | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| `with_latest_from` | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| **Time Operators (5)** | ✅ | ✅ | ✅ | ✅ | ✅ |
| `subscribe` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `subscribe_latest` | ✅ | ✅ | ✅ | ✅ | ❌ |
| `partition` | ✅ | ✅ | ✅ | ⚠️ | ❌ |
| `share` | ✅ | ✅ | ✅ | ✅ | ❌ |

**Legend:**
- ✅ Works without workarounds
- ⚠️ Requires alternative pattern (documented above)
- ❌ Not available

---

## Impact Assessment

##Tokio/smol/async-std users:** ✅ Zero impact - all operators work perfectly.

**WASM users:**
- ⚠️ 2 operators temporarily need workarounds (combine_latest, with_latest_from)
- ✅ All task spawning operators work (subscribe_latest, partition, share)
- **Fixed in v0.9.0** through interface consolidation

**Embassy users:**
- ⚠️ 2 operators temporarily need workarounds (combine_latest, with_latest_from) - **Fixed in v0.9.0**
- ❌ 3 operators permanently unavailable (subscribe_latest, partition, share) - **Cannot be fixed**

### Limitation Types

**Interface Inconsistencies (⚠️):**
- **Cause:** Overly restrictive trait bounds in current implementation
- **Impact:** Prevents compilation on WASM/Embassy despite runtime capability
- **Resolution:** Will be fixed in v0.9.0 through `Runtime` trait abstraction
- **Workaround quality:** Production-ready alternatives available (MergedStream)

**Fundamental Limitations (❌):**
- **Cause:** Embassy's static task allocation model (by design for no_std)
- **Impact:** Cannot spawn arbitrary futures with captured state
- **Resolution:** Cannot be fixed - architectural incompatibility
- **Workaround quality:** Manual patterns follow Embassy best practices

### Migration Path

**v0.9.0 changes:**
- ✅ `combine_latest`/`with_latest_from` will work on WASM/Embassy
- ✅ No breaking changes to existing code
- ❌ Task spawning operators remain Embassy-incompatible (by design)

These limitations will be resolved in v0.9.0 with workspace restructuring. No breaking changes required in user code - just additional operators will become available.

---

## Best Practices

### For Embassy/WASM Development

**1. Use MergedStream for complex state fusion:**
```rust
// Preferred pattern for sensor fusion, state aggregation, etc.
MergedStream::seed(initial_state)
    .merge_with(stream1, |val, state| update_state(val, state))
    .merge_with(stream2, |val, state| update_state(val, state))
```

**2. Use ordered_merge for simple merging:**
```rust
// When you need all items, not just latest
stream1.ordered_merge(vec![stream2, stream3])
```

**3. Add type annotations proactively:**
```rust
// In complex chains, add types early
.map_ordered(|item: EmbassyTimestamped<T>| { ... })
```

**4. Use manual polling for subscribe-like behavior:**
```rust
// Standard Embassy pattern
loop {
    match stream.next().await { ... }
}
```
### What WILL Be Fixed in v0.9.0

**Interface inconsistencies (⚠️):**
- ✅ `combine_latest`/`with_latest_from` work on WASM/Embassy
- ✅ Runtime-aware trait bounds via `Runtime` abstraction
- ✅ Perfect type inference in operator chains
- ✅ No workarounds needed for interface issues

**Compatibility:**
- ✅ Existing code keeps working
- ✅ Workaround patterns remain valid (just optional)
- ✅ No breaking changes to operator APIs

### What CANNOT Be Fixed

**Embassy task spawning (❌):**
- ❌ `subscribe_latest`/`partition`/`share` incompatible with Embassy
- ❌ Cannot spawn arbitrary futures in Embassy (by design)
- ✅ Documented manual patterns available

**This is not a bug** - it's a fundamental architectural difference between:
- **Dynamic runtimes** (Tokio/smol/async-std/WASM): Global spawn with captured state
- **Static runtime** (Embassy): Compile-time task allocation for no_std

See [FUTURE_ARCHITECTURE.md](./FUTURE_ARCHITECTURE.md) for v0.9.0 runtime abstraction details.ition tests in `fluxion-stream-time/tests/` for examples

---

## Future Resolution

All three limitations will be resolved in v0.9.0 through workspace restructuring. See [FUTURE_ARCHITECTURE.md](./FUTURE_ARCHITECTURE.md) for details.

**What will change:**
- ✅ `combine_latest`/`with_latest_from` will work in Embassy
- ✅ Perfect type inference in all chains (no annotations)
- ✅ `subscribe_latest`/`partition` will work in Embassy
- ✅ No workarounds needed

**What won't change:**
- ✅ Existing code keeps working
- ✅ Workaround patterns remain valid (just optional)
- ✅ No breaking changes to operator APIs
