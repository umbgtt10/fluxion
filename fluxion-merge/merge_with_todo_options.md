# MergedStream Integration with fluxion-stream - Design Options

> **Status**: TODO - Implementation postponed for later
> 
> **Decision**: Move `merge_with` from separate crate into `fluxion-stream` as a standard operator
>
> **Target API**:
> ```rust
> MergedStream::seed(Repository::new())
>     .merge_with(stream1, |item, state| state.from_testdata(item))
>     .merge_with(stream2, |item, state| state.from_testdata(item))
>     .merge_with(stream3, |item, state| state.from_testdata(item))
>     .combine_latest(vec![stream4], |_| true)  // Chains with other operators!
> ```
> 
> **Key Benefits**:
> - ✅ **NO TURBOFISH** - Type inference works perfectly
> - ✅ **Stateful** - `seed()` creates shared state, all `merge_with()` calls use it
> - ✅ **Chainable** - Works seamlessly with `combine_latest`, `with_latest_from`, etc.
> - ✅ **Clean** - 72+ characters of type parameters eliminated

## Goal
Enable chaining `merge_with` with other fluxion-stream operators:

```rust
MergedStream::seed(initial_state)
    .merge_with(stream1, process_fn1)
    .merge_with(stream2, process_fn2)
    .merge_with(stream3, process_fn3)
    .combine_latest(vec![stream4], |_| true)
    .with_latest_from(stream5, ...)
```

## Current State

### What Works
- ✅ `MergedStream` implements `Stream` trait
- ✅ Multiple `merge_with` calls can be chained
- ✅ Outputs `Sequenced<T>` or any `Timestamped` wrapper
- ✅ Tests pass with current API

### Current Limitations
- ❌ Cannot use with fluxion-stream operators (combine_latest, with_latest_from, etc.)
- ❌ Outputs generic `Item` type, not `StreamItem<T>` which fluxion-stream expects
- ❌ Not part of the fluxion-stream operator ecosystem

## Design Options

---

## Option 1: Make MergedStream Output StreamItem (RECOMMENDED)

### Concept
Change `MergedStream` to output `StreamItem<T>` instead of raw `Timestamped` types. This makes it compatible with all existing fluxion-stream operators.

### Implementation

```rust
// In fluxion-merge/Cargo.toml
[dependencies]
fluxion-core = { path = "../fluxion-core", version = "0.2.1" }

// In fluxion-merge/src/merged_stream.rs
use fluxion_core::StreamItem;

impl<S, State, T> Stream for MergedStream<S, State, T>
where
    S: Stream<Item = T>,
    T: Timestamped,
{
    type Item = StreamItem<T::Inner>;  // ← Key change!

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                Poll::Ready(Some(StreamItem::new(item.into_inner(), item.timestamp())))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

// Automatically implements all fluxion-stream extension traits!
impl<S, State, T> CombineLatestExt<T::Inner> for MergedStream<S, State, T>
where
    S: Stream<Item = T> + Send + Sync + 'static,
    T: Timestamped + Clone + Debug + Ord + Send + Sync + Unpin + CompareByInner + 'static,
    T::Inner: Clone + Debug + Ord + Send + Sync + 'static,
    T::Timestamp: Clone + Debug + Ord + Send + Sync,
{
    // Gets combine_latest, etc. for free
}
```

### Usage After Implementation

```rust
use fluxion_stream::{CombineLatestExt, WithLatestFromExt};
use fluxion_merge::MergedStream;

MergedStream::seed(initial_state)
    .merge_with(stream1, |item, state| { ... })
    .merge_with(stream2, |item, state| { ... })
    // Now works! ↓
    .combine_latest(vec![stream3], |_| true)
    .with_latest_from(stream4, |a, b| (a, b))
```

### Pros
- ✅ **Natural integration**: Works seamlessly with all fluxion-stream operators
- ✅ **No new API**: Leverages existing extension trait pattern
- ✅ **Type safety**: `StreamItem` provides consistent wrapper type
- ✅ **Unlimited chaining**: Can chain any number of `merge_with` calls
- ✅ **Minimal changes**: Only change output type in Stream impl

### Cons
- ⚠️ **Breaking change**: Existing tests need updates (output type changes from `Sequenced<T>` to `StreamItem<T>`)
- ⚠️ **Dependency**: Requires `fluxion-core` dependency in `fluxion-merge`

### Migration Impact
All existing code changes from:
```rust
let result: Sequenced<Repository> = merged_stream.next().await.unwrap();
```
To:
```rust
let result: StreamItem<Repository> = merged_stream.next().await.unwrap();
```

---

## Option 2: Create MergeWithExt Extension Trait

### Concept
Create a new extension trait in `fluxion-stream` that wraps `MergedStream`.

### Implementation

```rust
// In fluxion-stream/Cargo.toml
[dependencies]
fluxion-merge = { path = "../fluxion-merge", version = "0.2.1" }

// In fluxion-stream/src/merge_with.rs
pub trait MergeWithExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Timestamped + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn merge_stateful<State, NewStream, F>(
        self,
        initial_state: State,
        new_stream: NewStream,
        process_fn: F,
    ) -> impl Stream<Item = StreamItem<T>>
    where
        State: Send + Sync + 'static,
        NewStream: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
        F: FnMut(T::Inner, &mut State) -> T::Inner + Send + Sync + Clone + 'static;
}

impl<S, T> MergeWithExt<T> for S
where
    S: Stream<Item = StreamItem<T>> + Send + Sync + 'static,
    T: Timestamped + Clone + Debug + Ord + Send + Sync + Unpin + 'static,
{
    fn merge_stateful<State, NewStream, F>(
        self,
        initial_state: State,
        new_stream: NewStream,
        process_fn: F,
    ) -> impl Stream<Item = StreamItem<T>>
    {
        // Convert StreamItem back to Sequenced for MergedStream
        let self_as_sequenced = self.map(|item| Sequenced::with_timestamp(item.value, item.timestamp));
        let new_as_sequenced = new_stream.map(|item| Sequenced::with_timestamp(item.value, item.timestamp));
        
        MergedStream::seed(initial_state)
            .merge_with(self_as_sequenced, process_fn.clone())
            .merge_with(new_as_sequenced, process_fn)
            // Convert back to StreamItem
            .map(|seq| StreamItem::new(seq.into_inner(), seq.timestamp()))
    }
}
```

### Usage After Implementation

```rust
use fluxion_stream::{MergeWithExt, CombineLatestExt};

stream1
    .merge_stateful(initial_state, stream2, process_fn)
    .combine_latest(vec![stream3], |_| true)
```

### Pros
- ✅ **No breaking changes**: Existing `fluxion-merge` API unchanged
- ✅ **Separation of concerns**: fluxion-merge stays independent
- ✅ **Gradual adoption**: Users choose when to integrate

### Cons
- ❌ **Can't chain multiple merge_with**: Only works for two streams at once
- ❌ **Awkward API**: Different method name (`merge_stateful` vs `merge_with`)
- ❌ **Performance**: Extra map operations to convert between StreamItem/Sequenced
- ❌ **Complexity**: Two layers of abstraction

### Problem: No Chaining

This approach **cannot** support:
```rust
stream1
    .merge_stateful(state, stream2, fn2)
    .merge_with(stream3, fn3)  // ← DOESN'T WORK!
```

Because `merge_stateful` calls `MergedStream::seed()` internally, so each call creates a new state.

---

## Option 3: Hybrid Approach - Separate First Call

### Concept
Provide two methods: one for seeding, one for chaining.

### Implementation

```rust
// First merge includes seed
fn merge_with_state<State>(...) -> MergedStreamWrapper { ... }

// Subsequent merges reuse state
impl MergedStreamWrapper {
    fn merge_with(...) -> Self { ... }
}
```

### Usage

```rust
stream1
    .merge_with_state(initial_state, stream2, fn2)
    .merge_with(stream3, fn3)
    .merge_with(stream4, fn4)
```

### Pros
- ✅ Supports chaining
- ✅ No breaking changes to fluxion-merge

### Cons
- ❌ **Inconsistent API**: Two different method names
- ❌ **Confusing**: Users need to remember which to use when
- ❌ **Still can't mix**: Can't combine_latest then merge_with

---

## Recommended Solution: Option 1 with Stateful Chaining

### Final API Design

```rust
// Create stateful merger with initial state
MergedStream::seed(initial_state)
    // Chain multiple merge_with - all share the same state
    .merge_with(stream1, |item, state| process(item, state))
    .merge_with(stream2, |item, state| process(item, state))
    .merge_with(stream3, |item, state| process(item, state))
    // Now can chain with ANY fluxion-stream operator!
    .combine_latest(vec![stream4], |_| true)
    .with_latest_from(stream5, |a, b| (a, b))
    .take_latest_when(filter_stream, |x| x > 10)
```

**No turbofish needed anywhere!**

### Why This Works

1. **`seed()` establishes state type**:
   ```rust
   MergedStream::seed(Repository::new())  // State = Repository
   ```

2. **First `merge_with` infers all types**:
   ```rust
   .merge_with(stream1, |item: TestData, state| -> Repository { ... })
   //          ^^^^^^^ Stream<Item = StreamItem<TestData>>
   //                   ^^^^ Inferred from stream1
   //                                    ^^^^^^ Inferred from closure return
   ```

3. **Subsequent `merge_with` reuses same state**:
   - All calls share `Arc<Mutex<State>>`
   - No need to pass state again
   - Types flow through naturally

4. **Output is always `StreamItem<T>`**:
   - Compiler knows the wrapper type
   - No ambiguity between Sequenced/StreamItem/custom wrappers
   - Works with all fluxion-stream operators

### Implementation Details

```rust
// MergedStream outputs StreamItem (not raw Timestamped)
impl<S, State, T> Stream for MergedStream<S, State, T>
where
    S: Stream<Item = T>,
    T: Timestamped,
{
    type Item = StreamItem<T::Inner>;  // ← KEY: Fixed wrapper type
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().inner.poll_next(cx) {
            Poll::Ready(Some(item)) => {
                Poll::Ready(Some(StreamItem::new(
                    item.into_inner(),    // Unwrap value
                    item.timestamp()       // Preserve timestamp
                )))
            }
            // ...
        }
    }
}

// merge_with signature (no turbofish needed!)
pub fn merge_with<NewStream, F, InItem, OutItem>(
    self,
    new_stream: NewStream,
    process_fn: F,
) -> MergedStream<impl Stream<Item = StreamItem<OutItem>>, State, StreamItem<OutItem>>
where
    NewStream: Stream<Item = StreamItem<InItem>>,
    F: FnMut(InItem, &mut State) -> OutItem,
{
    // Implementation converts StreamItem -> process -> StreamItem
}
```

### Before vs After Comparison

#### Current (Hideous):
```rust
use fluxion_merge::MergedStream;
use fluxion_test_utils::Sequenced;

let mut merged_stream = MergedStream::seed(Repository::new())
    .merge_with::<_, _, Sequenced<TestData>, TestData, Sequenced<Repository>, Repository, u64>(
        //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //        7 TYPE PARAMETERS! 😱
        stream1,
        |item: TestData, state: &mut Repository| state.from_testdata(item),
    )
    .merge_with::<_, _, Sequenced<TestData>, TestData, Sequenced<Repository>, Repository, u64>(
        //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //        AGAIN! 😱😱
        stream2,
        |item: TestData, state: &mut Repository| state.from_testdata(item),
    );

let state: Sequenced<Repository> = merged_stream.next().await.unwrap();
//         ^^^^^^^^^^^^^^^^^^^^ Still need type annotation

// CANNOT do this:
// merged_stream.combine_latest(...) // ❌ Type mismatch!
```

#### After Option 1 (Beautiful):
```rust
use fluxion_stream::prelude::*;
use fluxion_stream::merge_with::MergedStream;

let mut merged_stream = MergedStream::seed(Repository::new())
    .merge_with(stream1, |item, state| state.from_testdata(item))
    //          ^^^^^^^ NO TURBOFISH! 🎉
    .merge_with(stream2, |item, state| state.from_testdata(item))
    //          ^^^^^^^ NO TURBOFISH! 🎉
    .combine_latest(vec![stream3], |_| true);
    //              ^^^^^^^^^^^^^^^^^^^^^^^^^^^ JUST WORKS! 🎊

let state: StreamItem<Repository> = merged_stream.next().await.unwrap();
//         ^^^^^^^^^^^^^^^^^^^^^^ Simpler type
```

**Turbofish reduction**: 144 characters → 0 characters per `merge_with` call!

### Why Option 1?

1. **Natural Composition**: `MergedStream` becomes a first-class fluxion-stream operator
2. **Consistent API**: Same patterns as other operators
3. **Maximum Flexibility**: Can chain in any order:
   ```rust
   stream1
       .merge_with(stream2, fn)
       .combine_latest(vec![stream3], |_| true)
       .merge_with(stream4, fn)  // ← Works!
   ```
4. **Type Safety**: `StreamItem` provides uniform wrapper type
5. **Clean Implementation**: No wrapper traits or conversions needed

### Migration Path

1. **Update fluxion-merge**:
   - Change `Stream::Item` from `Item` to `StreamItem<T::Inner>`
   - Add `fluxion-core` dependency
   - Update internal conversions

2. **Update tests**:
   - Change `Sequenced<T>` to `StreamItem<T>` in type annotations
   - Tests still work, just different output type

3. **Add to fluxion-stream** (optional):
   - No code changes needed!
   - Just document that `MergedStream` works with all operators
   - Could re-export `MergedStream::seed` for convenience

### Implementation Checklist

- [ ] Add `fluxion-core` dependency to `fluxion-merge/Cargo.toml`
- [ ] Import `StreamItem` in `merged_stream.rs`
- [ ] Update `Stream` impl to output `StreamItem<T::Inner>`
- [ ] Update all tests to use `StreamItem` instead of `Sequenced`
- [ ] Run full test suite
- [ ] Update documentation
- [ ] Add integration example showing chaining with fluxion-stream operators

---

## Open Questions

1. **Should we keep Sequenced support?**
   - Could have two impls: one for `StreamItem`, one for raw `Sequenced`
   - Probably not worth the complexity

2. **Turbofish syntax**?
   - Current API requires verbose type annotations
   - Could add helper methods for common cases
   - Future: investigate if trait aliases could help

3. **State management**?
   - Current: Arc<Mutex<State>>
   - Could optimize for single-threaded case
   - Good enough for now

4. **Error handling**?
   - Currently panics on lock failures
   - Could return Result from operators
   - Consider for v0.3

---

## Final Decision: Move to fluxion-stream (NOT keep as separate crate)

### Why NOT Keep Separate?

1. **It's fundamentally a stream operator** - like `combine_latest`, `with_latest_from`
2. **Separation creates friction** - different output types, extra imports
3. **No independence benefit** - already depends on fluxion-core and fluxion-ordered-merge
4. **State management isn't special** - other operators maintain state too
5. **Simpler architecture** - one crate for all stream operators

### Migration Plan

1. **Move code**: `fluxion-merge/src/merged_stream.rs` → `fluxion-stream/src/merge_with.rs`
2. **Update output**: Change `Stream::Item` to `StreamItem<T>` (Option 1)
3. **Add extension trait**: Follow existing `CombineLatestExt` pattern
4. **Move tests**: Integration tests go to `fluxion-stream/tests/`
5. **Remove crate**: Delete `fluxion-merge` entirely
6. **Update dependencies**: Remove from workspace

### Benefits After Migration

- ✅ Single import: `use fluxion_stream::prelude::*`
- ✅ Natural chaining with all operators
- ✅ Consistent `StreamItem` output type
- ✅ Simpler project structure
- ✅ No conversion overhead

---

## Timeline Estimate

**Full Migration (Recommended)**: ~3-5 hours
- 1 hour: Move code and update to StreamItem output
- 1 hour: Create extension trait following fluxion-stream patterns  
- 1 hour: Move and update all tests
- 1 hour: Update documentation and examples
- 30 min: Clean up workspace, remove fluxion-merge crate

**Option 1 Only (keeping separate crate)**: ~2-4 hours
- Less work but maintains unnecessary complexity
- Not recommended

---

## TODO Checklist

When implementing later:

### Phase 1: Core Changes
- [ ] Review this document and confirm approach
- [ ] Create feature branch: `feature/merge-with-integration`
- [ ] Move `fluxion-merge/src/merged_stream.rs` → `fluxion-stream/src/merge_with.rs`
- [ ] Update `Stream` impl to output `StreamItem<T::Inner>` instead of raw `Item`
- [ ] Update `merge_with` signature to work with `StreamItem` input/output
- [ ] Ensure state (`Arc<Mutex<State>>`) is properly shared across chained calls

### Phase 2: Extension Trait (if needed)
- [ ] Decide: Keep `MergedStream::seed()` or add `StreamExt::merge_with_state()`?
- [ ] If extension trait: Create `MergeWithExt` following existing patterns
- [ ] Ensure trait bounds allow chaining with other operators

### Phase 3: Tests
- [ ] Move tests: `fluxion-merge/tests/` → `fluxion-stream/tests/merge_with_tests.rs`
- [ ] Update all test type annotations: `Sequenced<T>` → `StreamItem<T>`
- [ ] **Remove all turbofish syntax** from test calls (should infer!)
- [ ] Add integration tests showing chaining with `combine_latest`, `with_latest_from`
- [ ] Verify no performance regression

### Phase 4: Integration
- [ ] Update `fluxion-stream/src/lib.rs`:
  - Add `pub mod merge_with;`
  - Export `MergedStream` in prelude
- [ ] Update `fluxion-stream/Cargo.toml` if new dependencies needed
- [ ] Move benchmarks: `fluxion-merge/benches/` → `fluxion-stream/benches/`
- [ ] Update benchmark code to use `StreamItem`

### Phase 5: Cleanup
- [ ] Remove `fluxion-merge` from workspace `Cargo.toml`
- [ ] Delete `fluxion-merge/` directory entirely
- [ ] Update main `README.md` - remove fluxion-merge references
- [ ] Update `fluxion-stream/README.md` - add merge_with documentation
- [ ] Add migration guide for existing users (if any)

### Phase 6: Documentation & Examples
- [ ] Document `MergedStream::seed()` with examples
- [ ] Document stateful chaining pattern
- [ ] Show before/after comparison (turbofish elimination)
- [ ] Add example combining merge_with + combine_latest + with_latest_from
- [ ] Update CHANGELOG with breaking changes note

### Phase 7: Validation
- [ ] Run full test suite: `cargo test`
- [ ] Run benchmarks: `cargo bench`
- [ ] Check all clippy warnings: `cargo clippy --all-targets`
- [ ] Verify documentation builds: `cargo doc --no-deps --open`
- [ ] Test in real-world scenario (if available)

### Success Criteria
- ✅ All tests pass without turbofish syntax
- ✅ Can chain `MergedStream` with `combine_latest`, `with_latest_from`, etc.
- ✅ Type inference works for all common cases
- ✅ No performance regression
- ✅ Documentation is clear and complete
