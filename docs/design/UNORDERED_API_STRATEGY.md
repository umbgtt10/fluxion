# Unordered API Strategy

This document explores strategies for adding an unordered (non-temporal) API alongside the existing ordered (temporal) API in Fluxion.

## Table of Contents

| Section | Description |
|---------|-------------|
| [Background](#background) | Current state and goals |
| [Strategy 1: Feature Flags](#strategy-1-feature-flags) | Compile-time switching between ordered/unordered |
| [Strategy 2: Generic Strategy Pattern](#strategy-2-generic-strategy-pattern-recommended) | Parameterized FluxionStream with merge strategy |
| [Strategy 3: Runtime Strategy](#strategy-3-runtime-strategy) | Runtime mode selection |
| [Strategy 4: Separate Extension Traits](#strategy-4-separate-extension-traits) | Timestamped vs Unordered trait namespaces |
| [Strategy 5: Method Suffix Convention](#strategy-5-method-suffix-convention) | `_ordered` vs `_unordered` methods |
| [Strategy 6: Separate Crates](#strategy-6-completely-separate-types-recommended-with-no-mixing-constraint) | **RECOMMENDED** - Private common crate + public variants |
| [Comparison Summary](#comparison-summary) | Side-by-side evaluation of all strategies |
| [Key Insight: Conversions Are Harmful](#key-insight-conversions-are-harmful) | Why mixing ordered/unordered destroys semantics |
| [Recommendation](#recommendation-strategy-6-separate-crates-with-private-common) | **Why Strategy 6 is the right choice** |
| [Implementation Roadmap](#implementation-roadmap-for-strategy-6) | Step-by-step migration plan |
| [Proof of Concept / Spike](#proof-of-concept--spike-strategy) | **How to validate the architecture** |
| [Benchmarking Strategy](#benchmarking-strategy) | **Performance validation and trade-off analysis** |
| [Testing Strategy](#phase-9-testing-strategy) | Test structure and code reduction |
| [Code Size Estimation](#code-size-estimation) | Expected lines of code impact |

---

## Background

Fluxion currently provides stream operators with **temporal ordering guarantees** via `ordered_merge`. This ensures events are processed in timestamp order across multiple streams. However, some use cases don't require ordering and would benefit from the performance gains of unordered processing.

### Current State

- All operators use `ordered_merge` internally
- Method names use `_ordered` suffix (e.g., `map_ordered`, `filter_ordered`)
- `FluxionStream<S>` is the single stream type
- `Timestamped` trait required for all stream items

### Goals

1. **Add unordered variants** of existing operators
2. **Maximize code reuse** between ordered/unordered implementations
3. **Maintain backward compatibility** with existing API
4. **Provide intuitive naming** and developer experience

---

## Strategy 1: Feature Flags

### Approach

Use Cargo feature flags to switch between ordered and unordered implementations at compile time.

```rust
// Cargo.toml
[features]
default = ["ordered"]
ordered = []
unordered = []
```

```rust
// In operator implementation
#[cfg(feature = "ordered")]
pub fn combine_latest<...>(...) -> impl Stream {
    // Uses ordered_merge
    streams.ordered_merge().filter_map(...)
}

#[cfg(feature = "unordered")]
pub fn combine_latest<...>(...) -> impl Stream {
    // Uses select_all (no ordering)
    futures::stream::select_all(streams).filter_map(...)
}
```

### User Experience

```toml
# User's Cargo.toml - choose at compile time
[dependencies]
fluxion = { version = "0.3", features = ["unordered"] }
```

```rust
// Code looks identical regardless of feature
stream.combine_latest(others, filter)
```

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐ | Confusing - can't use both in same project |
| Code Reuse | ⭐⭐⭐ | Operators duplicated with `#[cfg]` |
| Naming | ⭐⭐⭐⭐⭐ | No naming changes needed |
| Backward Compatibility | ⭐⭐⭐ | Default feature preserves behavior |

### Pros
- Zero API surface increase
- No naming conflicts
- Users choose once, use everywhere

### Cons
- **Cannot mix ordered and unordered in same project**
- Confusing for users who need both
- Testing both configurations is complex
- IDE/docs show only one variant

---

## Strategy 2: Generic Strategy Pattern (Recommended)

### Approach

Parameterize `FluxionStream` with a merge strategy type. The strategy determines how streams are combined internally.

```rust
// Strategy trait
pub trait MergeStrategy: Default + Clone + Send + Sync + 'static {
    fn merge<T, S>(streams: Vec<S>) -> Pin<Box<dyn Stream<Item = T> + Send>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Send + 'static;
}

// Strategy implementations
#[derive(Default, Clone)]
pub struct Ordered;

#[derive(Default, Clone)]
pub struct Unordered;

impl MergeStrategy for Ordered {
    fn merge<T, S>(streams: Vec<S>) -> Pin<Box<dyn Stream<Item = T> + Send>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Ord + Send + 'static,
    {
        Box::pin(streams.ordered_merge())
    }
}

impl MergeStrategy for Unordered {
    fn merge<T, S>(streams: Vec<S>) -> Pin<Box<dyn Stream<Item = T> + Send>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Send + 'static,
    {
        Box::pin(futures::stream::select_all(streams))
    }
}
```

```rust
// Parameterized FluxionStream with default
pub struct FluxionStream<S, M: MergeStrategy = Ordered> {
    inner: S,
    _marker: PhantomData<M>,
}

// Type aliases for ergonomics
pub type OrderedStream<S> = FluxionStream<S, Ordered>;
pub type UnorderedStream<S> = FluxionStream<S, Unordered>;
```

```rust
// Single implementation for all strategies
impl<S, M: MergeStrategy> FluxionStream<S, M> {
    pub fn combine_latest<...>(self, others: Vec<...>, filter: F) -> FluxionStream<impl Stream, M>
    where
        // constraints...
    {
        let streams = vec![self.inner, ...others];
        let merged = M::merge(streams);  // Strategy determines merge behavior
        FluxionStream::new(merged.filter_map(...))
    }
}
```

### User Experience

```rust
// Existing code unchanged (defaults to Ordered)
let result = stream.combine_latest(others, filter);

// Explicit ordered
let result: OrderedStream<_> = stream.combine_latest(others, filter);

// Opt into unordered
let result = stream.as_unordered().combine_latest(others, filter);

// Or explicit type
let result: UnorderedStream<_> = stream.into_unordered().combine_latest(others, filter);
```

### Conversion Methods

```rust
impl<S> FluxionStream<S, Ordered> {
    /// Convert to unordered mode. Loses temporal guarantees but may improve performance.
    pub fn as_unordered(self) -> FluxionStream<S, Unordered> {
        FluxionStream { inner: self.inner, _marker: PhantomData }
    }
}

impl<S> FluxionStream<S, Unordered> {
    /// Convert to ordered mode. Adds temporal ordering guarantees.
    pub fn as_ordered(self) -> FluxionStream<S, Ordered> {
        FluxionStream { inner: self.inner, _marker: PhantomData }
    }
}
```

### Method Naming with `_ordered` Suffix

The current `_ordered` suffix becomes **documentation of behavior**, not a naming conflict:

| Current Name | Meaning | Unordered Equivalent |
|--------------|---------|---------------------|
| `map_ordered` | Map preserving order | `map_ordered` (same name, different strategy) |
| `filter_ordered` | Filter preserving order | `filter_ordered` (same name, different strategy) |
| `combine_latest` | Combine with ordering | `combine_latest` (same name, different strategy) |

The `_ordered` suffix indicates the operation **respects the stream's ordering mode**, not that it forces ordering. This is semantically correct for both strategies:
- On `OrderedStream`: processes in timestamp order
- On `UnorderedStream`: processes in arrival order

**Alternative**: Rename to remove `_ordered` suffix entirely since the stream type now carries that information:

```rust
// Before
stream.map_ordered(|x| x * 2)
stream.filter_ordered(|x| x > 0)

// After (cleaner, strategy in type)
stream.map(|x| x * 2)      // behavior depends on stream type
stream.filter(|x| x > 0)   // behavior depends on stream type
```

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐⭐⭐ | Clear types, easy conversion |
| Code Reuse | ⭐⭐⭐⭐⭐ | Single implementation per operator |
| Naming | ⭐⭐⭐⭐ | Same names, type determines behavior |
| Backward Compatibility | ⭐⭐⭐⭐⭐ | Default type parameter preserves all existing code |

### Pros
- **Maximum code reuse** - single implementation per operator
- **Zero breaking changes** - default parameter preserves behavior
- **Type-safe** - can't accidentally mix strategies incorrectly
- **Clear mental model** - stream type carries ordering semantics
- **IDE-friendly** - one method, type annotations guide usage

### Cons
- More complex type signatures
- PhantomData adds slight complexity
- May need trait bounds adjustments
- **CRITICAL FLAW: Allows semantic errors through `.as_ordered()` / `.as_unordered()` conversions**
  - Mixing ordered/unordered in same chain creates stale timestamps
  - No compile-time prevention of invalid conversions
  - Users may accidentally lose ordering guarantees

**VERDICT: ❌ Don't use this approach if conversions are harmful**

---

## Strategy 3: Runtime Strategy

### Approach

Store the merge mode as runtime state in `FluxionStream`.

```rust
#[derive(Clone, Copy, Default)]
pub enum MergeMode {
    #[default]
    Ordered,
    Unordered,
}

pub struct FluxionStream<S> {
    inner: S,
    mode: MergeMode,
}

impl<S> FluxionStream<S> {
    pub fn with_mode(mut self, mode: MergeMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn unordered(self) -> Self {
        self.with_mode(MergeMode::Unordered)
    }

    pub fn combine_latest<...>(...) -> FluxionStream<impl Stream> {
        match self.mode {
            MergeMode::Ordered => {
                // ordered_merge implementation
            }
            MergeMode::Unordered => {
                // select_all implementation
            }
        }
    }
}
```

### User Experience

```rust
// Default ordered
let result = stream.combine_latest(others, filter);

// Explicit unordered
let result = stream.unordered().combine_latest(others, filter);

// Runtime choice
let mode = if config.needs_ordering { MergeMode::Ordered } else { MergeMode::Unordered };
let result = stream.with_mode(mode).combine_latest(others, filter);
```

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐⭐⭐⭐ | Simplest API, runtime flexibility |
| Code Reuse | ⭐⭐⭐ | Match statements in each operator |
| Naming | ⭐⭐⭐⭐⭐ | No changes needed |
| Backward Compatibility | ⭐⭐⭐⭐⭐ | Default mode preserves behavior |

### Pros
- Simplest user-facing API
- Runtime flexibility (can change based on config)
- No type complexity
- Easy to understand

### Cons
- **Runtime overhead** from match statements
- **Less type safety** - mode can change unexpectedly
- **Code duplication** - match in every operator
- Not idiomatic Rust (prefer compile-time when possible)

---

## Strategy 4: Separate Extension Traits

### Approach

Define separate extension traits for ordered and unordered operations.

```rust
// Ordered operations (current)
pub trait OrderedStreamExt<T>: Stream<Item = StreamItem<T>> {
    fn combine_latest_ordered(...) -> ...;
    fn map_ordered(...) -> ...;
    fn filter_ordered(...) -> ...;
}

// Unordered operations (new)
pub trait UnorderedStreamExt<T>: Stream<Item = StreamItem<T>> {
    fn combine_latest(...) -> ...;
    fn map(...) -> ...;
    fn filter(...) -> ...;
}

// Both implemented for FluxionStream
impl<S, T> OrderedStreamExt<T> for FluxionStream<S> where ... { }
impl<S, T> UnorderedStreamExt<T> for FluxionStream<S> where ... { }
```

### User Experience

```rust
use fluxion::OrderedStreamExt;    // Import timestamped trait
use fluxion::UnorderedStreamExt;  // Import unordered trait

// Ordered (explicit)
stream.combine_latest_ordered(others, filter)

// Unordered (explicit)
stream.combine_latest(others, filter)
```

### Method Naming Options

**Option A: Suffix on ordered (current behavior is "special")**
| Ordered | Unordered |
|---------|-----------|
| `combine_latest_ordered` | `combine_latest` |
| `map_ordered` | `map` |
| `filter_ordered` | `filter` |

**Option B: Suffix on unordered (ordered is default)**
| Ordered | Unordered |
|---------|-----------|
| `combine_latest` | `combine_latest_unordered` |
| `map` | `map_unordered` |
| `filter` | `filter_unordered` |

**Option C: Suffix on both (most explicit)**
| Ordered | Unordered |
|---------|-----------|
| `combine_latest_ordered` | `combine_latest_unordered` |
| `map_ordered` | `map_unordered` |
| `filter_ordered` | `filter_unordered` |

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐⭐ | Must import correct trait, naming conflicts possible |
| Code Reuse | ⭐⭐ | Full duplication of each operator |
| Naming | ⭐⭐⭐ | Clear but verbose |
| Backward Compatibility | ⭐⭐⭐⭐ | Existing code works with existing imports |

### Pros
- Very explicit - no ambiguity
- Can use both in same file (with care)
- No type system complexity

### Cons
- **Code duplication** - every operator implemented twice
- **Import management** - users must import correct trait
- **Name conflicts** if both traits imported
- **Maintenance burden** - changes must be mirrored

---

## Strategy 5: Method Suffix Convention

### Approach

Keep single `FluxionStream` type, add `_unordered` suffix to new methods.

```rust
impl<S> FluxionStream<S> {
    // Ordered (existing)
    pub fn combine_latest(...) -> ... { /* ordered_merge */ }
    pub fn map_ordered(...) -> ... { /* preserves order */ }

    // Unordered (new)
    pub fn combine_latest_unordered(...) -> ... { /* select_all */ }
    pub fn map_unordered(...) -> ... { /* no ordering */ }
}
```

### User Experience

```rust
// Ordered (default, existing)
stream.combine_latest(others, filter)
stream.map_ordered(|x| x * 2)

// Unordered (new, explicit)
stream.combine_latest_unordered(others, filter)
stream.map_unordered(|x| x * 2)
```

### Handling Existing `_ordered` Suffix

The current naming creates an inconsistency:

| Current | Unordered | Problem |
|---------|-----------|---------|
| `combine_latest` | `combine_latest_unordered` | ✓ Consistent |
| `map_ordered` | `map_unordered` | ✗ Asymmetric (why not just `map`?) |
| `filter_ordered` | `filter_unordered` | ✗ Asymmetric |

**Solution A: Rename existing methods (breaking change)**
```rust
// Remove _ordered suffix from existing
map_ordered → map
filter_ordered → filter

// Add _unordered for new
map_unordered
filter_unordered
```

**Solution B: Add aliases (non-breaking)**
```rust
// Keep existing
pub fn map_ordered(...) { ... }

// Add unsuffixed alias
pub fn map(...) { self.map_ordered(...) }

// Add unordered variant
pub fn map_unordered(...) { ... }
```

**Solution C: Embrace asymmetry**
- Document that `_ordered` suffix means "order-aware"
- `_unordered` suffix means "order-agnostic"
- Base name (`combine_latest`) defaults to ordered

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐⭐⭐ | Clear naming, discoverable |
| Code Reuse | ⭐⭐ | Full duplication of each operator |
| Naming | ⭐⭐⭐ | Verbose, existing `_ordered` creates asymmetry |
| Backward Compatibility | ⭐⭐⭐⭐⭐ | Additive only |

### Pros
- No type complexity
- Very explicit behavior
- Easy to discover both variants
- Fully backward compatible

### Cons
- **Code duplication** - every operator twice
- **Naming asymmetry** with existing `_ordered` suffix
- **API surface doubles**
- Verbose method names

---

## Strategy 6: Completely Separate Types (Recommended with No-Mixing Constraint)

### Approach

Define two entirely separate stream types with **no conversions between them**.

```rust
/// Stream with temporal ordering guarantees
pub struct OrderedStream<S> {
    inner: S,
}

/// Stream with arrival-order processing (no temporal guarantees)
pub struct UnorderedStream<S> {
    inner: S,
}

// Ordered operations require OrderedItem (includes Timestamped)
impl<S, T> OrderedStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: OrderedItem,  // = Timestamped + Clone + Send + Sync + 'static
{
    pub fn combine_latest(...) -> OrderedStream<impl Stream> {
        // Uses ordered_merge - requires timestamps
        let merged = streams.ordered_merge();
        OrderedStream::new(merged.filter_map(...))
    }

    pub fn map<F, R>(...) -> OrderedStream<impl Stream>
    where
        R: OrderedItem,
    {
        OrderedStream::new(self.inner.map(...))
    }
}

// Unordered operations only require UnorderedItem (no Timestamped!)
impl<S, T> UnorderedStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: UnorderedItem,  // = Clone + Send + Sync + 'static (NO Timestamped!)
{
    pub fn combine_latest(...) -> UnorderedStream<impl Stream> {
        // Uses select_all - no timestamps needed
        let merged = futures::stream::select_all(streams);
        UnorderedStream::new(merged.filter_map(...))
    }

    pub fn map<F, R>(...) -> UnorderedStream<impl Stream>
    where
        R: UnorderedItem,
    {
        UnorderedStream::new(self.inner.map(...))
    }
}

// NO .as_ordered() or .as_unordered() methods exist!
```

### Trait Bounds

```rust
/// Requirements for unordered streams (minimal)
pub trait UnorderedItem: Clone + Send + Sync + 'static {}

/// Requirements for ordered streams (adds temporal semantics)
pub trait OrderedItem: Timestamped + UnorderedItem {}

// Blanket implementations
impl<T> UnorderedItem for T where T: Clone + Send + Sync + 'static {}
impl<T> OrderedItem for T where T: Timestamped + Clone + Send + Sync + 'static {}
```

### User Experience

```rust
// Choose at stream creation - LOCKED IN forever
let ordered = OrderedStream::from_receiver(rx);
ordered
    .combine_latest(others, filter)  // OrderedStream
    .map(|x| x * 2)                  // OrderedStream
    .filter(|x| x > 0)               // OrderedStream

let unordered = UnorderedStream::from_receiver(rx);
unordered
    .combine_latest(others, filter)  // UnorderedStream
    .map(|x| x * 2)                  // UnorderedStream
    .filter(|x| x > 0)               // UnorderedStream

// Mixing is a COMPILE ERROR
let mixed = ordered.as_unordered();  // ❌ Method doesn't exist!
```

### "Conversion" Through Re-ingestion

The only way to switch modes is explicit re-ingestion:

```rust
// Want unordered from ordered? Extract values through a channel
let (tx, rx) = unbounded_channel();

ordered_stream.subscribe_async(move |item| {
    tx.send(item.into_inner());  // Extract value, discard timestamp
});

// Start fresh in unordered mode
let unordered = UnorderedStream::from_receiver(rx);
```

This makes the **semantic loss explicit** - you're deliberately discarding ordering.

### Evaluation

| Criteria | Score | Notes |
|----------|-------|-------|
| User Experience | ⭐⭐⭐⭐⭐ | Crystal clear, explicit dependency choice |
| Code Reuse | ⭐⭐⭐⭐⭐ | Private common crate = zero duplication |
| Naming | ⭐⭐⭐⭐⭐ | Same method names, type determines behavior |
| Backward Compatibility | ⭐⭐⭐⭐⭐ | Keep `fluxion-stream` as re-export facade |
| Type Safety | ⭐⭐⭐⭐⭐ | Compiler prevents mixing |
| Compile Time | ⭐⭐⭐⭐⭐ | Only compiles what you use |
| Binary Size | ⭐⭐⭐⭐⭐ | Smaller - no dead code |

### Pros
- **Zero code duplication** - shared logic in private common crate
- **Compile-time prevention of mixing** - impossible to accidentally lose semantics
- **Faster builds** - only compile the variant you need
- **Smaller binaries** - tree shaking eliminates unused variant
- **No PhantomData** - simpler type signatures
- **No default type parameters** - explicit is better
- **Clearer error messages** - `OrderedItem` vs `UnorderedItem`
- **Better optimization** - each crate can be specialized independently
- **Prevents semantic bugs** - can't have stale timestamps
- **Simple mental model** - choose crate, get guarantees
- **Dependency-level separation** - choice visible in Cargo.toml

### Cons
- More crates to manage (but fits existing workspace pattern)
- Users must choose upfront (but this is actually a **pro** - forces clear thinking)
- Need to publish two crates instead of one (minor overhead)

### Why This Beats Strategy 2

| Aspect | Strategy 2 (Generic) | Strategy 6 (Separate Crates) |
|--------|---------------------|------------------------------|
| Conversion | `.as_ordered()` / `.as_unordered()` | ❌ **None** (prevents bugs) |
| Type Params | `FluxionStream<S, M = Ordered>` | `OrderedStream<S>` / `UnorderedStream<S>` |
| PhantomData | Required | Not needed |
| Error Messages | "bound `M: MergeStrategy` not satisfied" | "bound `T: OrderedItem` not satisfied" |
| Semantic Safety | ⚠️ Can mix and create stale timestamps | ✅ Impossible to mix |
| Code Reuse | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ (via private common crate) |
| Compile Time | ⭐⭐ (always both) | ⭐⭐⭐⭐⭐ (only what you use) |
| Binary Size | ⭐⭐ (includes both) | ⭐⭐⭐⭐⭐ (tree-shaken) |
| Dependency Clarity | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ (visible in Cargo.toml) |
| Clarity | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## Comparison Summary

| Strategy | Code Reuse | UX | Naming | Compat | Safety | Build Time | Binary Size | Recommended |
|----------|------------|-----|--------|--------|--------|------------|-------------|-------------|
| 1. Feature Flags | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | No |
| 2. Generic Strategy | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐ | No - allows harmful conversions |
| 3. Runtime Strategy | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐ | ⭐⭐ | No - runtime overhead |
| 4. Separate Traits | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | No |
| 5. Method Suffix | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | Fallback |
| 6. Separate Crates | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | **YES** |

### Key Insight: Conversions Are Harmful

The critical realization is that **allowing conversions between ordered and unordered destroys semantic guarantees**:

```rust
// Strategy 2 allows this (BAD):
let ordered = stream.combine_latest(...);  // Timestamps from merge
let unordered = ordered.as_unordered();    // Timestamps now meaningless
let ordered_again = unordered.as_ordered(); // Timestamps are STALE!
```

**Strategy 6 prevents this** by making separate types with **no conversion methods**.

---

## Recommendation: Strategy 6 (Separate Crates with Private Common)

### Why This Is The Right Choice

1. **Zero Code Duplication**: Private `fluxion-stream-common` crate contains all shared logic
2. **Prevents Semantic Bugs**: Impossible to accidentally mix ordered/unordered and create stale timestamps
3. **Compile-Time Safety**: Type system enforces that ordered requires `OrderedItem` (with timestamps)
4. **Faster Builds**: Only compile the variant you actually use
5. **Smaller Binaries**: Dead code elimination removes unused variant
6. **Simpler Types**: No generic parameters, no PhantomData
7. **Clear Mental Model**: Choose crate, get guarantees
8. **Better Error Messages**: `OrderedItem` vs `UnorderedItem` is clearer than strategy bounds
9. **Dependency-Level Clarity**: Choice is explicit in `Cargo.toml`
10. **Explicit Re-ingestion**: Switching modes requires intentional channel-based re-ingestion

### Best of Both Worlds

Strategy 6 with separate crates achieves what seemed impossible:

| Aspect | Strategy 2 | Strategy 6 (Separate Crates) |
|--------|-----------|-----------------------------|
| Code Duplication | None ✅ | None ✅ (via private crate) |
| Type Safety | Weak ⚠️ (allows conversions) | Strong ✅ (prevents mixing) |
| Compile Time | Slow ⚠️ (always both) | Fast ✅ (only what you use) |
| Binary Size | Large ⚠️ (includes both) | Small ✅ (tree-shaken) |

**You get correctness AND efficiency AND zero duplication.**

### Crate Structure Fits Existing Pattern

You already have 6 published crates in the workspace:
- `fluxion` (rx)
- `fluxion-core`
- `fluxion-ordered-merge`
- `fluxion-stream` → **becomes facade/deprecated**
- `fluxion-exec`
- `fluxion-test-utils`

Adding 3 more follows the same modular philosophy:
- `fluxion-stream-common` (private)
- `fluxion-stream-ordered` (published)
- `fluxion-stream-unordered` (published)

---

## Implementation Roadmap for Strategy 6

### Phase 1: Define Trait Bounds
### Phase 1: Define Trait Bounds

```rust
// Add to fluxion-core/src/lib.rs

/// Minimal requirements for unordered stream processing
pub trait UnorderedItem: Clone + Send + Sync + 'static {}

/// Requirements for ordered (temporal) stream processing
pub trait OrderedItem: Timestamped + UnorderedItem {}

// Blanket implementations
impl<T> UnorderedItem for T where T: Clone + Send + Sync + 'static {}
impl<T> OrderedItem for T where T: Timestamped + Clone + Send + Sync + 'static {}
```

**Backward Compatibility**: Deprecate existing `FluxionItem`:
```rust
#[deprecated(since = "0.3.0", note = "Use `OrderedItem` instead")]
pub trait FluxionItem: OrderedItem {}
impl<T: OrderedItem> FluxionItem for T {}
```

### Phase 2: Create Separate Stream Types

```rust
// Add to fluxion-stream/src/lib.rs

/// Stream with temporal ordering guarantees
pub struct OrderedStream<S> {
    inner: S,
}

/// Stream with arrival-order processing
pub struct UnorderedStream<S> {
    inner: S,
}

// Factory methods
impl<T> OrderedStream<UnboundedReceiverStream<StreamItem<T>>>
where
    T: OrderedItem,
{
    pub fn from_unbounded_receiver(rx: UnboundedReceiver<T>) -> Self {
        Self::new(UnboundedReceiverStream::new(rx.map(StreamItem::Value)))
    }
}

impl<T> UnorderedStream<UnboundedReceiverStream<StreamItem<T>>>
where
    T: UnorderedItem,
{
    pub fn from_unbounded_receiver(rx: UnboundedReceiver<T>) -> Self {
        Self::new(UnboundedReceiverStream::new(rx.map(StreamItem::Value)))
    }
}
```

**Backward Compatibility**: Add type alias for existing code:
```rust
#[deprecated(since = "0.3.0", note = "Use `OrderedStream` instead")]
pub type FluxionStream<S> = OrderedStream<S>;
```

### Phase 3: Implement Operators for Both Types

```rust
// fluxion-stream-ordered/src/operators/combine_latest.rs
use fluxion_stream_common::combine_latest::combine_latest_impl;

impl<S, T> OrderedStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: OrderedItem + Ord,
{
    pub fn combine_latest<IS, F>(
        self,
        others: Vec<IS>,
        filter: F,
    ) -> OrderedStream<impl Stream<Item = StreamItem<CombinedState<T>>>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        F: Fn(&[T]) -> bool + Send + Sync + 'static,
    {
        let streams = vec![self.inner]
            .into_iter()
            .chain(others.into_iter().map(|s| s.into_stream()))
            .collect();

        let merged = streams.ordered_merge();  // Timestamp ordering
        OrderedStream::new(combine_latest_impl(merged, filter))  // Shared logic!
    }
}
```

```rust
// fluxion-stream-unordered/src/operators/combine_latest.rs
use fluxion_stream_common::combine_latest::combine_latest_impl;

impl<S, T> UnorderedStream<S>
where
    S: Stream<Item = StreamItem<T>>,
    T: UnorderedItem,  // NO Ord, NO Timestamped!
{
    pub fn combine_latest<IS, F>(
        self,
        others: Vec<IS>,
        filter: F,
    ) -> UnorderedStream<impl Stream<Item = StreamItem<CombinedState<T>>>>
    where
        IS: IntoStream<Item = StreamItem<T>>,
        F: Fn(&[T]) -> bool + Send + Sync + 'static,
    {
        let streams = vec![self.inner]
            .into_iter()
            .chain(others.into_iter().map(|s| s.into_stream()))
            .collect();

        let merged = select_all(streams);  // Arrival order
        UnorderedStream::new(combine_latest_impl(merged, filter))  // Same shared logic!
    }
}
```

### Phase 7: Update Workspace Cargo.toml

```toml
[workspace]
members = [
    "fluxion",
    "fluxion-core",
    "fluxion-ordered-merge",
    "fluxion-stream-common",    # NEW - private
    "fluxion-stream-ordered",   # NEW - published
    "fluxion-stream-unordered", # NEW - published
    "fluxion-stream",           # Deprecated facade
    "fluxion-exec",
    "fluxion-test-utils",
]

[workspace.dependencies]
# ... existing ...
fluxion-stream-common = { path = "fluxion-stream-common" }
fluxion-stream-ordered = { version = "0.4.0", path = "fluxion-stream-ordered" }
fluxion-stream-unordered = { version = "0.4.0", path = "fluxion-stream-unordered" }
```

### Phase 8: Migration Path

### Phase 8: Migration Path

**For existing code (uses fluxion-stream):**
```toml
# Before
[dependencies]
fluxion-stream = "0.3.0"
```

```toml
# After - explicit choice
[dependencies]
fluxion-stream-ordered = "0.4.0"  # Or fluxion-stream-unordered
```

```rust
// Code remains the same, just change the type name
// Before
use fluxion_stream::FluxionStream;
let stream = FluxionStream::from_unbounded_receiver(rx);

// After
use fluxion_stream_ordered::OrderedStream;
let stream = OrderedStream::from_unbounded_receiver(rx);
```

**For new unordered code:**
```toml
[dependencies]
fluxion-stream-unordered = "0.4.0"
```

```rust
use fluxion_stream_unordered::UnorderedStream;
let stream = UnorderedStream::from_unbounded_receiver(rx);
stream.combine_latest(others, filter)  // Same API, different guarantees!
```

### Phase 9: Testing Strategy

**Current Test Structure** (~9,400 lines across 26 files):
```
fluxion-stream/tests/
├── combine_latest_tests.rs (526 lines)
├── combine_latest_error_tests.rs (228 lines)
├── combine_with_previous_tests.rs (353 lines)
├── combine_with_previous_error_tests.rs (182 lines)
└── ... (similar pattern for all operators)
```

**New Test Structure with Separate Crates:**

#### Shared Test Helpers (fluxion-stream-common)

```rust
// fluxion-stream-common/src/testing.rs (or tests/helpers.rs)
// Reusable test logic that works for any stream type

/// Generic test that works for both ordered and unordered
pub async fn test_combine_latest_basic<S, Factory>(
    factory: Factory,
) -> anyhow::Result<()>
where
    S: Stream<Item = StreamItem<Sequenced<i32>>> + Unpin,
    Factory: Fn(UnboundedReceiver<Sequenced<i32>>) -> S,
{
    let (tx1, rx1) = unbounded_channel();
    let (tx2, rx2) = unbounded_channel();

    let mut stream = factory(rx1).combine_latest(vec![rx2], |_| true);

    tx1.send(Sequenced::with_timestamp(1, 1))?;
    tx2.send(Sequenced::with_timestamp(2, 2))?;

    let result = unwrap_stream(&mut stream, 100).await;
    assert!(result.is_value());

    Ok(())
}

/// Generic error propagation test
pub async fn test_combine_latest_error_propagation<S, Factory>(
    factory: Factory,
) -> anyhow::Result<()>
where
    S: Stream<Item = StreamItem<Sequenced<i32>>> + Unpin,
    Factory: Fn(UnboundedReceiver<Sequenced<i32>>) -> S,
{
    // Error handling logic - independent of ordering
}
```

#### Ordered-Specific Tests (fluxion-stream-ordered)

```rust
// fluxion-stream-ordered/tests/combine_latest_tests.rs (~200 lines)
use fluxion_stream_common::testing::*;
use fluxion_stream_ordered::OrderedStream;

#[tokio::test]
async fn test_combine_latest_basic() {
    test_combine_latest_basic(|rx| OrderedStream::from_unbounded_receiver(rx))
        .await
        .unwrap();
}

// Ordering-specific test
#[tokio::test]
async fn test_combine_latest_respects_timestamp_order() {
    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Send out-of-order by timestamp
    tx1.send(Sequenced::with_timestamp(1, 5))?;  // timestamp 5
    tx2.send(Sequenced::with_timestamp(2, 3))?;  // timestamp 3 (earlier!)

    // Should emit in timestamp order (3 before 5)
    let first = unwrap_stream(&mut result, 100).await;
    assert_eq!(first.timestamp(), 3);  // Earlier timestamp first!
}
```

```rust
// fluxion-stream-ordered/tests/combine_latest_error_tests.rs (~100 lines)
use fluxion_stream_common::testing::*;

#[tokio::test]
async fn test_error_propagation() {
    test_combine_latest_error_propagation(|rx| OrderedStream::from_unbounded_receiver(rx))
        .await
        .unwrap();
}

// No additional ordering-specific error tests needed
```

#### Unordered-Specific Tests (fluxion-stream-unordered)

```rust
// fluxion-stream-unordered/tests/combine_latest_tests.rs (~200 lines)
use fluxion_stream_common::testing::*;
use fluxion_stream_unordered::UnorderedStream;

#[tokio::test]
async fn test_combine_latest_basic() {
    test_combine_latest_basic(|rx| UnorderedStream::from_unbounded_receiver(rx))
        .await
        .unwrap();
}

// Arrival-order specific test
#[tokio::test]
async fn test_combine_latest_respects_arrival_order() {
    let (tx1, stream1) = test_channel();
    let (tx2, stream2) = test_channel();

    let mut result = stream1.combine_latest(vec![stream2], |_| true);

    // Send in specific arrival order
    tx1.send(item_with_value(1))?;  // Arrives first
    tx2.send(item_with_value(2))?;  // Arrives second

    let first = unwrap_stream(&mut result, 100).await;
    // No timestamp guarantees - just first to arrive wins
}
```

```rust
// fluxion-stream-unordered/tests/combine_latest_error_tests.rs (~100 lines)
use fluxion_stream_common::testing::*;

#[tokio::test]
async fn test_error_propagation() {
    test_combine_latest_error_propagation(|rx| UnorderedStream::from_unbounded_receiver(rx))
        .await
        .unwrap();
}
```

### Test File Breakdown Per Operator

For each operator (e.g., `combine_latest`):

**fluxion-stream-common/src/testing.rs**:
- Generic test helpers (~400 lines for all operators combined)
- Reusable across both variants

**fluxion-stream-ordered/tests/**:
- `combine_latest_tests.rs` (~250 lines)
  - Calls shared helpers (~50 lines)
  - Ordering-specific tests (~200 lines)
- `combine_latest_error_tests.rs` (~100 lines)
  - Calls shared error helpers (~50 lines)
  - Ordering-specific error tests (~50 lines)

**fluxion-stream-unordered/tests/**:
- `combine_latest_tests.rs` (~200 lines)
  - Calls shared helpers (~50 lines)
  - Arrival-order tests (~150 lines)
- `combine_latest_error_tests.rs` (~100 lines)
  - Calls shared error helpers (~50 lines)
  - Unordered-specific tests (~50 lines)

### Total Files Per Variant

**Per operator in each variant:**
- ✅ 2 test files (not 3): `_tests.rs` + `_error_tests.rs`
- ✅ Shared logic in common crate eliminates duplication

**Total across 9 operators:**

| Crate | Test Files | Approx Lines |
|-------|-----------|--------------|
| `fluxion-stream-common` | 1 (helpers) | ~400 |
| `fluxion-stream-ordered` | 18 (9 × 2) | ~3,150 |
| `fluxion-stream-unordered` | 18 (9 × 2) | ~2,700 |
| **Total** | **37 files** | **~6,250 lines** |

**Compared to current:**
- Current: 26 files, ~9,400 lines
- New: 37 files, ~6,250 lines
- **Net reduction: ~3,150 lines (~33% less)** due to shared helpers!

### Answer: 2 Files Per Operator (Not 3)

**Yes, exactly 2 test files per variant:**
1. `<operator>_tests.rs` - functional tests
2. `<operator>_error_tests.rs` - error propagation tests

The third component (shared helpers) lives in `fluxion-stream-common` and is reused by both variants.

---

## Code Size Estimation

### Current Monolithic Approach
```
fluxion-stream/         ~4,100 lines
```

### Separate Crates Approach
```
fluxion-stream-common/      ~2,500 lines (all shared operator logic)
fluxion-stream-ordered/     ~700 lines (thin wrappers + ordered merge)
fluxion-stream-unordered/   ~600 lines (thin wrappers + select_all)
fluxion-stream/             ~50 lines (deprecated facade)
──────────────────────────────────────
Total:                      ~3,850 lines
```

**Result: ~250 lines LESS than current monolithic approach** due to better factoring!

---

## Proof of Concept / Spike Strategy

Before committing to the full migration, validate the architecture with a focused proof-of-concept.

### Recommended Spike: 3 Operators (Priority Order)

#### 1. **`combine_latest`** (FIRST - Critical Validation)

**Why start here:**
- **THE KEY OPERATOR** - exercises ordered vs unordered merge (the entire point!)
- Multi-stream merging validates the core architecture difference
- Uses `ordered_merge` (ordered) vs `select_all` (unordered)
- Rich test suite (754 lines) provides comprehensive validation
- Most complex operator - if this works, simpler ones will too

**What it proves:**
- ✅ `fluxion-stream-common` shared logic works
- ✅ `OrderedStream` correctly uses `ordered_merge`
- ✅ `UnorderedStream` correctly uses `select_all`
- ✅ Trait bounds (`OrderedItem` vs `UnorderedItem`) compile and work
- ✅ Test helpers can be extracted and reused
- ✅ Error propagation works in both variants
- ✅ **The entire architecture is viable**

**Estimated effort:** 8-10 hours

**Success criteria:**
```rust
// Ordered variant - timestamp ordering preserved
OrderedStream::combine_latest(...) // uses ordered_merge()

// Unordered variant - arrival order
UnorderedStream::combine_latest(...) // uses select_all()

// Shared logic extracted
fluxion_stream_common::combine_latest_impl(...) // reused by both
```

---

#### 2. **`map_ordered`** (SECOND - Validation of Simplest Pattern)

**Why second:**
- Simplest operator (~200 lines)
- Single stream (no merge complexity)
- Validates basic infrastructure for simple cases
- Different pattern than `combine_latest` (transform vs merge)

**What it proves:**
- ✅ Simple operators work with shared pattern
- ✅ Single-stream operators integrate cleanly
- ✅ Pattern scales from complex to simple
- ✅ No surprises with basic transforms

**Estimated effort:** 3-4 hours

---

#### 3. **`scan_ordered`** (THIRD - Stateful Validation)

**Why third:**
- Stateful operator (accumulator pattern)
- Different from both merge and transform patterns
- Validates shared logic works for stateful operations
- Moderate complexity (274 lines)

**What it proves:**
- ✅ Stateful operators can be extracted to common
- ✅ Pattern works beyond just merge-based operators
- ✅ State management doesn't create issues
- ✅ No architectural gaps remain

**Estimated effort:** 4-5 hours

---

### Spike Timeline (3 Days)

**Day 1: Infrastructure + `combine_latest`**
1. Create crate structure (common, ordered, unordered)
2. Add `OrderedItem`/`UnorderedItem` to fluxion-core
3. Extract `combine_latest` core logic to common
4. Implement thin wrappers in ordered/unordered
5. Migrate 5-7 key tests with shared helpers
6. Run test suites

**Success checkpoint:** `combine_latest` works in both variants, tests pass

---

**Day 2: `map_ordered` + Deeper Validation**
1. Extract `map_ordered` to common
2. Implement wrappers
3. Migrate tests
4. Run full test suite for both operators
5. Measure code reduction vs duplication

**Success checkpoint:** Simple pattern validated, confidence increasing

---

**Day 3: `scan_ordered` + Final Validation**
1. Extract stateful logic to common
2. Implement wrappers
3. Migrate tests
4. Run all tests
5. Document findings and decision

**Success checkpoint:** All patterns validated, clear path forward

---

### Alternative: Minimal Spike (Just `combine_latest`)

If you want the **absolute minimum** to decide:

**Just migrate `combine_latest` (1-2 days)**

This single operator proves the entire concept because:
- It's the most complex multi-stream operator
- It exercises the ordered vs unordered difference (the whole point!)
- Its test suite is comprehensive (754 lines)
- Other operators are simpler variations

**Estimated effort:** 8-10 hours

**Decision point:** If `combine_latest` works cleanly, you can confidently proceed with full migration.

---

### Spike Success Criteria (Go/No-Go Decision)

After the spike, you should be able to answer:

**✅ Architecture works if:**
- All tests pass for spiked operators
- Code reduction is visible (at least 20-30%)
- Shared helpers eliminate duplication
- No major compilation issues with trait bounds
- Ordered uses `ordered_merge`, unordered uses `select_all`
- Test migration is straightforward

**🚫 Architecture needs revision if:**
- Trait bounds cause widespread compilation issues
- Shared logic requires too much generic complexity
- Tests can't be easily shared between variants
- Code duplication is actually worse
- Performance issues emerge

---

## Full Implementation Roadmap (Post-Spike)

Assuming spike succeeds, here's the migration order for remaining operators:

### Phase 1: Proof of Concept (Spike)
- `combine_latest` ✅
- `map_ordered` ✅
- `scan_ordered` ✅

### Phase 2: Simple Operators (Low Risk)
- `filter_ordered` - similar to `map_ordered`
- `distinct_until_changed` - single stream, stateful
- `distinct_until_changed_by` - variant of above

### Phase 3: Medium Complexity
- `with_latest_from` - similar to `combine_latest`
- `emit_when` - conditional emission
- `take_while_with` - conditional termination

### Phase 4: Complex Operators
- `merge_with` - builder pattern, stateful
- `take_latest_when` - sampling logic
- `combine_with_previous` - window-based

### Phase 5: Unordered-Exclusive Operators

**Wall-clock time-based operators (only make sense for unordered):**

```rust
// fluxion-stream-unordered ONLY
impl<S, T> UnorderedStream<S> {
    /// Suppresses rapid arrivals based on wall-clock time
    pub fn debounce(self, duration: Duration) -> UnorderedStream<...> {
        // Uses tokio::time::sleep
        // Emits only if no new item arrives within duration
    }

    /// Rate limits emissions based on wall-clock time
    pub fn throttle(self, duration: Duration) -> UnorderedStream<...> {
        // Ensures minimum duration between emissions
    }

    /// Time-based windowing using wall-clock time
    pub fn window_time(self, duration: Duration) -> UnorderedStream<...> {
        // Buffers items arriving within time window
    }
}
```

**Why unordered-only?**

| Aspect | Ordered (Event-Time) | Unordered (Wall-Clock) |
|--------|---------------------|------------------------|
| Time meaning | Event timestamps (historical, replayed) | Wall-clock arrival time (real-time) |
| `debounce(100ms)` | Ambiguous - 100ms of event time? | Clear - 100ms of wall-clock time |
| Use case | Temporal reasoning, event sourcing | Real-time processing, latency-sensitive |
| Timer source | Would need watermarks (complex) | `tokio::time::sleep` (natural) |
| Semantic fit | ❌ Breaks temporal consistency | ✅ Natural for arrival-order processing |

**Key insight:** Wall-clock operations belong in the unordered variant because:
1. Unordered = real-time, arrival-order processing
2. Ordered = temporal reasoning with event timestamps from data
3. Mixing wall-clock timers with event timestamps breaks ordering guarantees
4. This is a feature of separate crates - variant-specific operators!

**Documentation note:**
```rust
// This will NOT compile - debounce doesn't exist on OrderedStream
let stream = OrderedStream::from_receiver(rx);
stream.debounce(Duration::from_millis(100));  // ❌ Compile error!

// This works - debounce is exclusive to UnorderedStream
let stream = UnorderedStream::from_receiver(rx);
stream.debounce(Duration::from_millis(100));  // ✅ Wall-clock debouncing
```

### Phase 6: Cleanup
- Deprecate `fluxion-stream`
- Update all documentation
- Update examples
- Publish `0.5.0` (or appropriate version)

---

## Benchmarking Strategy

### Goals

1. **Validate performance assumptions** - Prove unordered is actually faster
2. **Quantify trade-offs** - Measure how much faster for different scenarios
3. **Guide users** - Provide data-driven recommendations for when to use each variant
4. **Continuous monitoring** - Track performance regressions

---

### Benchmark Structure (Separate Crates Approach)

#### Shared Benchmark Scenarios (fluxion-stream-common)

```rust
// fluxion-stream-common/benches/scenarios.rs
// Reusable benchmark scenarios that work for any stream type

pub struct BenchScenario {
    pub name: &'static str,
    pub stream_count: usize,
    pub items_per_stream: usize,
    pub payload_size: usize,
}

pub const SCENARIOS: &[BenchScenario] = &[
    // Small workload
    BenchScenario { name: "small", stream_count: 3, items_per_stream: 100, payload_size: 16 },
    // Medium workload
    BenchScenario { name: "medium", stream_count: 5, items_per_stream: 1000, payload_size: 64 },
    // Large workload
    BenchScenario { name: "large", stream_count: 10, items_per_stream: 10000, payload_size: 128 },
    // Many streams
    BenchScenario { name: "wide", stream_count: 20, items_per_stream: 500, payload_size: 32 },
];

/// Generic benchmark helper
pub async fn bench_combine_latest_scenario<F, S>(
    scenario: &BenchScenario,
    stream_factory: F,
) -> u64
where
    F: Fn(usize) -> S,
    S: Stream<Item = StreamItem<Sequenced<Vec<u8>>>> + Unpin,
{
    // Shared benchmark logic
    let start = Instant::now();

    // Create streams using the factory
    let streams: Vec<_> = (0..scenario.stream_count)
        .map(|_| stream_factory(scenario.items_per_stream))
        .collect();

    // Run the operator
    let mut combined = streams[0].combine_latest(streams[1..].to_vec(), |_| true);

    while let Some(item) = combined.next().await {
        black_box(item);
    }

    start.elapsed().as_micros() as u64
}
```

#### Ordered Benchmarks (fluxion-stream-ordered)

```rust
// fluxion-stream-ordered/benches/combine_latest_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use fluxion_stream_common::benches::scenarios::{SCENARIOS, bench_combine_latest_scenario};
use fluxion_stream_ordered::OrderedStream;

fn bench_ordered_combine_latest(c: &mut Criterion) {
    let mut group = c.benchmark_group("ordered/combine_latest");

    for scenario in SCENARIOS {
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.name),
            scenario,
            |b, scenario| {
                b.to_async(Runtime::new().unwrap()).iter(|| async {
                    bench_combine_latest_scenario(scenario, |size| {
                        OrderedStream::from_items(make_ordered_items(size))
                    }).await
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_ordered_combine_latest);
criterion_main!(benches);
```

#### Unordered Benchmarks (fluxion-stream-unordered)

```rust
// fluxion-stream-unordered/benches/combine_latest_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use fluxion_stream_common::benches::scenarios::{SCENARIOS, bench_combine_latest_scenario};
use fluxion_stream_unordered::UnorderedStream;

fn bench_unordered_combine_latest(c: &mut Criterion) {
    let mut group = c.benchmark_group("unordered/combine_latest");

    for scenario in SCENARIOS {
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.name),
            scenario,
            |b, scenario| {
                b.to_async(Runtime::new().unwrap()).iter(|| async {
                    bench_combine_latest_scenario(scenario, |size| {
                        UnorderedStream::from_items(make_unordered_items(size))
                    }).await
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_unordered_combine_latest);
criterion_main!(benches);
```

---

### Comparison Benchmark (Side-by-Side)

Create a dedicated comparison benchmark that runs both variants:

```rust
// fluxion-stream-ordered/benches/compare_with_unordered.rs
// This benchmark depends on both crates for direct comparison

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use fluxion_stream_ordered::OrderedStream;
use fluxion_stream_unordered::UnorderedStream;

fn compare_combine_latest(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison/combine_latest");

    let sizes = [100, 1000, 10000];

    for &size in &sizes {
        // Ordered variant
        group.bench_with_input(
            BenchmarkId::new("ordered", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Benchmark ordered variant
                });
            },
        );

        // Unordered variant
        group.bench_with_input(
            BenchmarkId::new("unordered", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Benchmark unordered variant
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, compare_combine_latest);
criterion_main!(benches);
```

---

### Integration with Spike Plan

#### During Spike (Day 1: `combine_latest`)

**Step 5a: Add Benchmarks**

After implementing `combine_latest` for both variants, add benchmarks:

1. Extract shared scenario to `fluxion-stream-common/benches/scenarios.rs`
2. Create `ordered/benches/combine_latest_bench.rs`
3. Create `unordered/benches/combine_latest_bench.rs`
4. Run both: `cargo bench --package fluxion-stream-ordered --package fluxion-stream-unordered`

**Expected results to validate:**
```
ordered/combine_latest/small      time: [450 µs 460 µs 470 µs]
unordered/combine_latest/small    time: [180 µs 185 µs 190 µs]  <- 2.5x faster!

ordered/combine_latest/medium     time: [5.2 ms 5.4 ms 5.6 ms]
unordered/combine_latest/medium   time: [2.1 ms 2.2 ms 2.3 ms]  <- 2.4x faster!
```

**Success criteria:**
- ✅ Unordered is measurably faster (1.5-3x expected)
- ✅ Benchmarks run without errors
- ✅ Shared scenarios work for both variants

**🚨 Red flags:**
- Unordered is slower (architecture problem!)
- Ordered performance regressed vs current implementation
- High variance in results

---

#### Benchmark Output Format

Use criterion's comparison features to generate a trade-off table:

```bash
# Run comparison
cargo bench --bench compare_with_unordered

# Generate HTML report
open target/criterion/comparison/combine_latest/report/index.html
```

**Automated Trade-off Table Generation:**

```rust
// scripts/generate_tradeoff_table.rs
// Parse criterion output and generate markdown table

fn generate_tradeoff_table() -> String {
    r#"
| Operator | Scenario | Ordered | Unordered | Speedup | Trade-off |
|----------|----------|---------|-----------|---------|-----------|
| combine_latest | small (3×100) | 460 µs | 185 µs | 2.49x | No ordering guarantee |
| combine_latest | medium (5×1K) | 5.4 ms | 2.2 ms | 2.45x | No ordering guarantee |
| combine_latest | large (10×10K) | 58 ms | 23 ms | 2.52x | No ordering guarantee |
| map_ordered | small | 120 µs | 115 µs | 1.04x | Minimal (single stream) |
| scan_ordered | medium | 3.2 ms | 2.8 ms | 1.14x | Minimal (stateful) |
    "#.to_string()
}
```

---

### Expected Performance Characteristics

Based on the architecture differences:

| Operator | Expected Speedup | Reason |
|----------|-----------------|--------|
| `combine_latest` | **2-3x faster** | Avoids heap-based ordered merge |
| `with_latest_from` | **2-3x faster** | Same - uses select_all vs ordered_merge |
| `merge_with` | **2-3x faster** | Builder pattern still benefits from unordered |
| `map_ordered` | **1.05-1.1x** | Single stream - minimal ordering overhead |
| `filter_ordered` | **1.05-1.1x** | Single stream |
| `scan_ordered` | **1.1-1.3x** | Stateful but no merge |
| `debounce` | **N/A** | **Unordered-only** - wall-clock operation |
| `throttle` | **N/A** | **Unordered-only** - wall-clock operation |

**Key insight:** Multi-stream operators see the biggest gains. Wall-clock operators only exist in unordered variant.

---

### Continuous Benchmarking

**Add to CI pipeline:**

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  pull_request:
    paths:
      - 'fluxion-stream-*/**'

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: |
          cargo bench --package fluxion-stream-ordered
          cargo bench --package fluxion-stream-unordered

      - name: Compare with baseline
        run: |
          # Compare against main branch
          # Fail if performance regresses >10%
```

**Store results:**
- Commit criterion reports to `docs/benchmarks/`
- Track trend over time
- Alert on regressions

---

### Trade-off Documentation

After benchmarking, update user-facing docs with a decision table:

```markdown
## When to Use Ordered vs Unordered

| Your Scenario | Recommendation | Expected Performance | Available Operators |
|---------------|----------------|---------------------|---------------------|
| Multi-stream combining (3+ streams) | **Unordered** if ordering doesn't matter | 2-3x faster | All shared operators |
| Event sourcing / audit logs | **Ordered** (correctness required) | Baseline | All shared operators |
| High-throughput analytics | **Unordered** | 2-3x faster | All shared operators |
| Single-stream transforms | **Either** (minimal difference) | <10% difference | All shared operators |
| Real-time dashboards | **Unordered** (latency matters) | 2-3x faster | All shared + debounce/throttle |
| Temporal causality required | **Ordered** (only option) | Baseline | All shared operators |
| Wall-clock debouncing/throttling | **Unordered** (only option) | N/A | debounce, throttle, window_time |
| Historical data replay | **Ordered** (event timestamps) | Baseline | All shared operators |
```

---

### Spike Day 1 Updated Timeline

**With benchmarking integrated:**

1. ✅ Infrastructure setup (2 hours)
2. ✅ Implement `combine_latest` logic (4 hours)
3. ✅ Migrate tests (2 hours)
4. **🆕 Add benchmarks (1.5 hours)**
   - Extract shared scenarios
   - Create ordered bench
   - Create unordered bench
5. **🆕 Run benchmarks & validate (0.5 hours)**
   - Verify unordered is faster
   - Check for regressions
   - Document results

**Total: 10 hours (was 8-10 hours)**

**Decision point:** If benchmarks show unordered is slower, investigate before proceeding!

---

// tests/unordered/combine_latest.rs
#[tokio::test]
async fn test_combine_latest_emits() {
    test_combine_latest_emits_impl::<Unordered>().await
}
```

**Pros:**
- Clear separation in test output
- No extra dependencies
- Easy to add strategy-specific tests

**Cons:**
- More boilerplate
- Two files per test

### Option 3: Strategy-Specific Test Suites

Some behaviors differ between strategies:

```rust
// Shared tests (both strategies)
mod common_tests {
    // combine_latest emits when all inputs present
    // filter removes items matching predicate
    // map transforms values
}

// Ordered-only tests
mod ordered_tests {
    // Events processed in timestamp order
    // Late events are reordered correctly
    // Temporal guarantees maintained
}

// Unordered-only tests
mod unordered_tests {
    // Events processed in arrival order
    // No reordering overhead
    // Performance benchmarks
}
```

**Recommendation:** Combine Options 1 and 3:
- Parameterized tests for shared behavior (~80% of tests)
- Strategy-specific tests for ordering guarantees (~20%)
- This does NOT double the test count, approximately +30%

---

## Code Organization Options

### Option A: Strategy in Core, Single Crate

```
fluxion-core/
  src/
    lib.rs
    merge_strategy.rs      # NEW: MergeStrategy trait
    ordered.rs             # NEW: Ordered impl
    unordered.rs           # NEW: Unordered impl

fluxion-stream/
  src/
    fluxion_stream.rs      # FluxionStream<S, M = Ordered>
    combine_latest.rs      # Single impl using M::merge()
    ...
```

**Pros:**
- Minimal workspace changes
- Clear where strategy lives
- Easy imports

**Cons:**
- Core crate grows

### Option B: Separate Strategy Crate

```
fluxion-core/
fluxion-strategy/           # NEW CRATE
  src/
    lib.rs
    merge_strategy.rs
    ordered.rs
    unordered.rs

fluxion-stream/
  Cargo.toml               # depends on fluxion-strategy
```

**Pros:**
- Clean separation of concerns
- Strategy can evolve independently
- Optional dependency for users who don't need both

**Cons:**
- More crates to manage
- More complex dependency graph
- Probably overkill

### Option C: Feature-Gated in Existing Crates (Recommended)

```
fluxion-core/
  src/
    lib.rs
    merge_strategy.rs       # Always available

fluxion-ordered-merge/      # Already exists, provides Ordered
fluxion-merge/              # Already exists, provides Unordered base

fluxion-stream/
  Cargo.toml
    [features]
    default = ["ordered"]
    ordered = ["fluxion-ordered-merge"]
    unordered = []          # Uses fluxion-merge
```

**Pros:**
- Leverages existing crate structure
- `fluxion-ordered-merge` already provides ordered merging
- `fluxion-merge` already provides unordered merging
- Users can opt out of ordering entirely (smaller binary)

**Cons:**
- Feature flags add complexity
- Conditional compilation in operators

### Option D: All-in-One with Internal Modules

```
fluxion-stream/
  src/
    lib.rs
    strategy/
      mod.rs                # pub trait MergeStrategy
      ordered.rs            # pub struct Ordered
      unordered.rs          # pub struct Unordered
    operators/
      combine_latest.rs     # impl<M: MergeStrategy>
      ...
    fluxion_stream.rs       # FluxionStream<S, M>
```

**Pros:**
- Self-contained
- Easy to understand
- No cross-crate complexity

**Cons:**
- fluxion-stream grows larger
- Strategy not reusable outside

---

## Test Organization (Recommended)

### Structure: One Subfolder Per Operator

Each operator gets its own test subfolder with files created **as needed**. Not all operators require all file types - only create files that have meaningful tests.

```
fluxion-stream/
  tests/
    combine_latest/
      mod.rs
      combine_latest_tests.rs               # Parameterized success tests (both strategies)
      combine_latest_error_tests.rs         # Parameterized error tests (both strategies)
      combine_latest_ordered_tests.rs       # Ordered-specific success tests
      combine_latest_ordered_error_tests.rs # Ordered-specific error tests
      combine_latest_unordered_tests.rs     # Unordered-specific success tests
      combine_latest_unordered_error_tests.rs # Unordered-specific error tests

    with_latest_from/
      mod.rs
      with_latest_from_tests.rs
      with_latest_from_error_tests.rs
      with_latest_from_ordered_tests.rs
      # ... other files as needed

    filter_ordered/
      mod.rs
      filter_ordered_tests.rs
      # Simple operator - may only need basic tests

    # ... one folder per operator
```

### File Types and Their Purpose

| File Pattern | Purpose | When to Create |
|--------------|---------|----------------|
| `<op>_tests.rs` | Parameterized success tests for both strategies | Always (core behavior) |
| `<op>_error_tests.rs` | Parameterized error tests for both strategies | When operator has error paths |
| `<op>_ordered_tests.rs` | Ordered-specific success tests | When ordering affects behavior |
| `<op>_ordered_error_tests.rs` | Ordered-specific error tests | When ordering affects error handling |
| `<op>_unordered_tests.rs` | Unordered-specific success tests | When unordered has unique behavior |
| `<op>_unordered_error_tests.rs` | Unordered-specific error tests | When unordered has unique error cases |

### Scaling by Operator Complexity

The 1:N relationship between source files and test files is **self-documenting**:

| Operator Complexity | Test Files Created |
|---------------------|-------------------|
| Simple, no strategy differences | `_tests.rs` only |
| Has error handling | `_tests.rs`, `_error_tests.rs` |
| Has ordered-specific behavior | + `_ordered_tests.rs` |
| Has ordered-specific errors | + `_ordered_error_tests.rs` |
| Has unordered-specific behavior | + `_unordered_tests.rs` |
| Full complexity | All 6 file types |

### Key Principles

1. **Files created as needed** - No empty placeholder files
2. **Self-documenting** - If a file exists, there's a reason
3. **Flexible** - Each operator gets exactly what it needs
4. **Discoverable** - Looking at a folder reveals what's tested and what edge cases exist
5. **Error handling is first-class** - Separate error test files reflect Fluxion's emphasis on error propagation

### Example: Parameterized Test File

```rust
// tests/combine_latest/combine_latest_tests.rs

use rstest::rstest;
use fluxion_stream::{Ordered, Unordered, MergeStrategy};

#[rstest]
#[case::ordered(Ordered)]
#[case::unordered(Unordered)]
#[tokio::test]
async fn test_emits_when_all_streams_have_values<M: MergeStrategy>(#[case] _strategy: M) {
    // Test runs twice: once for Ordered, once for Unordered
    // Implementation identical for both strategies
}

#[rstest]
#[case::ordered(Ordered)]
#[case::unordered(Unordered)]
#[tokio::test]
async fn test_handles_stream_completion<M: MergeStrategy>(#[case] _strategy: M) {
    // Another parameterized test
}
```

### Example: Strategy-Specific Test File

```rust
// tests/combine_latest/combine_latest_ordered_tests.rs

//! Tests specific to ordered combine_latest behavior.
//! These tests verify temporal ordering guarantees.

#[tokio::test]
async fn test_events_processed_in_timestamp_order() {
    // Only makes sense for ordered strategy
    // Verifies late events are reordered correctly
}

#[tokio::test]
async fn test_maintains_causality_across_streams() {
    // Ordered-specific: ensures event A before event B
    // if A's timestamp < B's timestamp
}

```rust
// tests/combine_latest_tests.rs

mod common {
    // Tests that apply to both strategies (parameterized)
}

mod ordered {
    // Tests specific to ordered behavior
}

mod unordered {
    // Tests specific to unordered behavior
}
```

**Pros:**
- All tests for an operator in one file
- Clear structure
- Easy to see coverage

---

## Summary of Recommendations

| Aspect | Recommendation |
|--------|----------------|
| **When to implement** | Hybrid - Infrastructure now, operators incrementally |
| **Testing approach** | Parameterized tests (~80%) + strategy-specific (~20%) |
| **Code organization** | Option A or D - Strategy in core or stream crate |
| **Test organization** | Option C - Single file with modules |
| **Test count impact** | ~30% increase, NOT doubled |

### Suggested Timeline

1. **v0.3.x**: Add `MergeStrategy` trait, keep everything else unchanged
2. **v0.4.x**: Migrate operators one-by-one, add parameterized tests
3. **v0.5.x**: Complete migration, add unordered-specific optimizations
4. **v1.0**: Consider renaming `_ordered` methods, finalize API

This approach minimizes risk, spreads effort, and allows course correction based on user feedback.

---

## Conclusion

The **Generic Strategy Pattern** provides the best balance of code reuse, user experience, and backward compatibility. It allows Fluxion to offer both ordered and unordered APIs without duplicating operator implementations, while maintaining a clean and intuitive interface.

The existing `_ordered` suffix can be retained initially (it documents that the operation is ordering-aware) and optionally deprecated in future versions once users are comfortable with the type-based approach.

The hybrid implementation approach (infrastructure now, operators incrementally) combined with parameterized testing ensures manageable effort while maintaining quality.

---

## Documentation Strategy

When the unordered API is implemented, documentation should follow the **augment, don't duplicate** principle.

### Key Principle

The operators are the same; only the merge strategy differs. Document it once, note the strategy, move on. **Don't create parallel documentation** - that doubles maintenance burden.

### Recommended Changes to `FLUXION_OPERATOR_SUMMARY.md`

#### 1. Add Ordering Modes Section (Near Top)

Add a new section after the introduction, before the Quick Reference Table:

```markdown
## Ordering Modes

Fluxion operators support two ordering modes:

| Mode | Type | Use When |
|------|------|----------|
| **Ordered** (default) | `FluxionStream<S, Ordered>` | Temporal order matters |
| **Unordered** | `FluxionStream<S, Unordered>` | Performance over order |

\`\`\`rust
// Default: ordered processing
let result = stream.combine_latest(others, filter);

// Opt into unordered for performance
let result = stream.as_unordered().combine_latest(others, filter);
\`\`\`

> **Note:** All operators listed below work identically in both modes.
> The difference is internal merge strategy, not behavior.
```

#### 2. Extend Quick Reference Table

Add a column indicating unordered support:

| Operator | Category | Purpose | Unordered? |
|----------|----------|---------|------------|
| `ordered_merge` | Combining | Merge streams temporally | ✅ (uses `select_all`) |
| `combine_latest` | Combining | Combine latest | ✅ |
| `map_ordered` | Transform | Transform items | ✅ |
| ... | ... | ... | ✅ |

#### 3. Per-Operator Notes (Only When Behavior Differs)

Most operators behave identically in both modes. Only add notes where unordered has meaningful differences:

```markdown
#### `ordered_merge`
...
> **Ordering modes:**
> - **Ordered**: Uses timestamp-based reordering (heap merge)
> - **Unordered**: Uses `select_all` (arrival order, lower overhead)
```

#### 4. Add "Choosing a Mode" to Common Patterns Section

```markdown
### Choosing Ordered vs Unordered

| Scenario | Recommended |
|----------|-------------|
| Causality matters (event A before B) | Ordered |
| Multi-source aggregation with timestamps | Ordered |
| High-throughput, order irrelevant | Unordered |
| Single-source transforms | Either (no difference) |
| Latency-sensitive pipelines | Unordered |
| Event sourcing / audit logs | Ordered |
```

### What NOT to Do

❌ **Don't create `FLUXION_OPERATOR_SUMMARY_UNORDERED.md`** - parallel docs drift apart

❌ **Don't duplicate every operator section** - adds maintenance burden

❌ **Don't add "Unordered" subsections under each operator** - clutters the doc

### Minimal Touch Points

When implementing unordered support, only these docs need updates:

1. **`FLUXION_OPERATOR_SUMMARY.md`** - Add ordering modes section, table column, choice guide
2. **`README.md`** - Brief mention in features section
3. **API rustdocs** - Add note to `FluxionStream` type documentation
4. **`CHANGELOG.md`** - Document the feature addition

The `UNORDERED_API_STRATEGY.md` document itself should be moved to `docs/design/` as architectural decision documentation, not user-facing reference.
