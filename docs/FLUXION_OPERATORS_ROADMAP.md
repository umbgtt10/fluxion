# Fluxion Operators Roadmap

This document tracks future operator additions and runtime capability enhancements.

## Current Status (v0.6.13)

**Implemented:** 27 operators across 5 runtimes
- ✅ All 27 operators on std runtimes (Tokio, smol, async-std, WASM)
- ✅ 25/27 operators on Embassy (embedded/no_std)
- ⏳ 2 operators pending TaskSpawner abstraction (subscribe_latest, partition)

## Version 0.9.0 - Complete Embassy Integration 🎯

**Status:** 🚀 **Planned** - The killer feature that sets Fluxion apart

### TaskSpawner Abstraction
Mirrors the proven `Timer` trait pattern for task spawning across all runtimes.

**Enables:**
- ✅ `subscribe_latest` on Embassy with spawner injection
- ✅ `partition` on Embassy with spawner injection
- ✅ All 27 operators work everywhere (servers to microcontrollers)

**Competitive Advantage:**
- RxRust: ❌ Locked into Tokio, no embedded support
- Other reactive libs: ❌ std-only
- Embassy ecosystem: ❌ No full-featured reactive streams
- **Fluxion v0.9.0**: ✅ Industry first - complete reactive streams on embedded systems

**Implementation:**
- TaskSpawner trait with GlobalTaskSpawner and EmbassyTaskSpawner implementations
- Convenience APIs (subscribe_latest() for global runtimes, subscribe_latest_with_embassy() for Embassy)
- Zero performance penalty, single operator implementation

---

## Status Legend
- 🚀 **Planned** - Scheduled for specific release
- 💭 **Considering** - Under evaluation
- 📝 **Research** - Investigating feasibility
- ⏸️ **Deferred** - Low priority, future consideration

---

## Medium Priority (Version 0.7.0+)

### `buffer` 💭
**Collect items into batches**

```rust
let buffered = stream.buffer(trigger_stream);
```

**Use case**: Batch processing, windowing
**Complexity**: Medium - requires internal buffering
**RxJS equivalent**: `buffer`

---

### `window` 💭
**Group items into sub-streams**

```rust
let windowed = stream.window(Duration::from_secs(5));
```

**Use case**: Time-based grouping, sliding windows
**Complexity**: High - requires nested streams
**RxJS equivalent**: `window`

---

## Lower Priority (Version 0.8.0+)

### `retry` 📝
**Retry failed operations**

```rust
let with_retry = stream.retry(3);
```

**Use case**: Error recovery, resilience
**Complexity**: High - requires error handling integration
**RxJS equivalent**: `retry`

---

### `merge_map` ⏸️
**Flat map with concurrency control**

```rust
let flat_mapped = stream.merge_map(|item| async_operation(item));
```

**Use case**: Parallel async operations
**Complexity**: Very High - requires concurrency management
**RxJS equivalent**: `mergeMap` / `flatMap`

---

### `switch_map` ⏸️
**Cancel previous async operations**

```rust
let switched = stream.switch_map(|item| async_operation(item));
```

**Use case**: Search as you type, cancellation
**Complexity**: Very High - requires cancellation tokens
**RxJS equivalent**: `switchMap`

---

### `concat_map` ⏸️
**Sequential async operations**

```rust
let sequential = stream.concat_map(|item| async_operation(item));
```

**Use case**: Sequential processing, ordered execution
**Complexity**: High - requires queuing
**RxJS equivalent**: `concatMap`

---

## Research Phase 1.X.0

### `group_by` 📝
**Group items by key into sub-streams**

**Use case**: Multi-tenant processing, categorization
**Complexity**: Very High - requires dynamic stream creation

---

### `zip` 📝
**Combine items pairwise from multiple streams**

**Use case**: Synchronization, pairing
**Complexity**: Medium - requires buffering from all streams

---

### `race` 📝
**Emit from whichever stream emits first**

**Use case**: Timeouts, fallbacks
**Complexity**: Medium - requires cancellation

---

### `take_until` 💭
**Take items until notifier stream emits**

```rust
let bounded = stream.take_until(stop_signal);
```

**Use case**: Graceful shutdown, cancellation, bounded processing
**Complexity**: Medium - requires coordination between streams
**RxJS equivalent**: `takeUntil`

---

### `reduce` 💭
**Aggregate all items into a single final value**

```rust
let total = stream.reduce(0, |acc, item| acc + item.value);
```

**Use case**: Aggregation, accumulation, final computation
**Complexity**: Low - similar to scan but emits only final value
**RxJS equivalent**: `reduce`

---

### `finalize` 📝
**Execute cleanup on stream completion, error, or cancellation**

```rust
let with_cleanup = stream.finalize(|| cleanup_resources());
```

**Use case**: Resource cleanup, metrics finalization, logging
**Complexity**: Medium - requires tracking all termination paths
**RxJS equivalent**: `finalize`

---

### `first` / `last` 📝
**Emit only the first or last item from a stream**

```rust
let first_item = stream.first();
let last_item = stream.last();
```

**Use case**: Single value extraction, initialization values
**Complexity**: Low - `first` is trivial, `last` requires buffering
**RxJS equivalent**: `first`, `last`

---

## Community Requests

This section will track operator requests from the community.

**How to request an operator:**
1. Open an issue on GitHub with the `operator-request` label
2. Describe the use case
3. Provide example code showing desired behavior
4. Link to equivalent operators in other libraries (RxJS, Reactor, etc.)

---

## Implementation Guidelines

When implementing new operators, follow these principles:

### 1. **Temporal Ordering**
All operators must preserve or correctly handle temporal ordering via sequence numbers.

### 2. **Type Safety**
Leverage Rust's type system for compile-time correctness. Avoid panics in public APIs.

### 3. **Documentation Standard**
Every operator must have:
- Clear description
- "Why use this?" section
- Complete runnable example
- Use cases list
- "See Also" cross-references
- Doc tests that compile and pass

### 4. **Testing Standard**
Every operator requires:
- Unit tests for basic functionality
- Integration tests for operator chaining
- Permutation tests for ordering guarantees
- Error condition tests
- Doc tests for examples

### 5. **Performance**
Operators should be:
- Zero-copy where possible
- Minimal allocation
- Benchmarked against alternatives

---

## Criteria for Operator Inclusion

An operator is added to Fluxion when it meets ALL of these criteria:

✅ **Common use case** - Solves a frequently encountered problem
✅ **Cannot be easily composed** - Not trivially built from existing operators
✅ **Well-defined semantics** - Clear, unambiguous behavior
✅ **Temporal ordering** - Works correctly with ordering guarantees
✅ **Maintains quality bar** - Meets documentation and testing standards

---

## Contributing New Operators

Interested in implementing an operator? See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

**Quick steps:**
1. Open an issue proposing the operator
2. Get feedback on design
3. Implement with tests and docs
4. Submit PR

We welcome contributions! 🎉

---

## See Also

- **[Operators Summary](FLUXION_OPERATOR_SUMMARY.md)** - Currently implemented operators
- **[ROADMAP.md](../ROADMAP.md)** - Overall project roadmap
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - How to contribute
