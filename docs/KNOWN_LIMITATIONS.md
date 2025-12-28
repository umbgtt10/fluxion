# Fluxion Known Limitations & Alternatives

**Last Updated:** December 28, 2025
**Status:** Current limitations in v0.7.0

---

## Overview

Fluxion currently has three known limitations affecting specific runtimes (Embassy/WASM). All limitations have documented workarounds and will be resolved in the planned workspace restructuring (v0.9.0).

**Summary:**
- ✅ **25/27 operators** work in all 5 runtimes
- ⚠️ **2 operators** (combine_latest, with_latest_from) require workarounds in Embassy/WASM
- ⚠️ **Type inference** occasionally requires explicit annotations when chaining
- ⚠️ **Task spawning operators** (subscribe_latest, partition) unavailable in Embassy

---

## Limitation 1: Combination Operators in Embassy/WASM

### Problem

`combine_latest` and `with_latest_from` do not work in single-threaded environments (Embassy, WASM).

```rust
// ❌ Fails in Embassy/WASM
use fluxion_stream::CombineLatestExt;

stream1.combine_latest(
    vec![stream2, stream3],
    |state| state.values().len() == 3
)
// Error: cannot be sent between threads safely
```

### Root Cause

These operators use `Arc<Mutex<State>>` for shared state management, which requires `Send + Sync` bounds. Embassy and WASM run on single-threaded executors where streams are `!Send`.

**Technical Details:**
```rust
// Current trait signature
fn combine_latest<IS>(self, others: Vec<IS>, ...)
    -> impl Stream<Item = StreamItem<CombinedState<...>>> + Send + Sync
where
    IS::Stream: Send + Sync + 'static,  // ← Embassy can't satisfy this
```

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

## Limitation 3: Task Spawning Operators in Embassy

### Problem

`subscribe_latest` and `partition` are unavailable in Embassy because they require task spawning.

```rust
// ❌ Not available in Embassy
stream.subscribe_latest(|item| async move {
    process(item).await;
});
```

### Root Cause

**Different execution models:**
- **Tokio/smol/async-std/WASM:** Global spawn model (`FluxionTask::spawn()`)
- **Embassy:** Factory model (requires injected `embassy_executor::Spawner`)

These operators currently use `FluxionTask` internally, which doesn't work in Embassy.

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
        match item {
            StreamItem::Value(val) if predicate(&val) => {
                tx_true.send(val).await.ok();
            }
            StreamItem::Value(val) => {
                tx_false.send(val).await.ok();
            }
            StreamItem::Error(e) => {
                // Handle error
            }
        }
    }
});

// Use rx_true and rx_false as separate streams
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

**Legend:**
- ✅ Works without workarounds
- ⚠️ Requires alternative pattern (documented above)
- ❌ Not available

---

## Impact Assessment

### Who Is Affected?

**Most users (Tokio/smol/async-std):** No impact, all operators work perfectly.

**WASM users:** 2 operators need alternatives (combine_latest, with_latest_from).

**Embassy users:** 4 operators need alternatives (combine_latest, with_latest_from, subscribe_latest, partition).

### Workaround Quality

All workarounds provide equivalent functionality:
- ✅ `MergedStream` is production-ready (used in embassy-sensors example)
- ✅ Manual polling patterns are standard Embassy practice
- ✅ No functionality is truly missing, just requires more code

### Migration Path

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

### For Tokio/smol/async-std Development

**No special considerations** - all operators work as documented. Type annotations rarely needed.

---

## Documentation References

- **MergedStream API:** See [fluxion-stream README](../fluxion-stream/README.md#mergedstream)
- **Embassy Example:** See [embassy-sensors](../examples/embassy-sensors/) for production patterns
- **Type Annotations Guide:** See composition tests in `fluxion-stream-time/tests/` for examples

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
