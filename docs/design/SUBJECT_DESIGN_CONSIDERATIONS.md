# Subject Design Considerations

This document explores the design considerations for implementing a Subject in the Fluxion ecosystem.

## What is a Subject?

A Subject is a hybrid primitive that acts as both:
- **Observable/Stream** - Can be subscribed to (supports multiple subscribers)
- **Observer/Sink** - Can receive values via push (`.next(value)`, `.error()`, `.complete()`)

## Key Design Questions

### 1. Relationship to Existing Primitives

**Current Fluxion Architecture:**
- `FluxionStream` - wraps any `Stream<Item = StreamItem<T>>`
- Channel-based construction (`IntoFluxionStream` trait)
- Temporal ordering via `HasTimestamp` trait

**Subject Requirements:**
- Provide a **sender interface** (push values)
- Provide a **FluxionStream** (subscribe/consume)
- Handle **multiple subscribers** (broadcast)
- Maintain **temporal ordering** across all subscribers

### 2. Timestamp Assignment Strategy

**Critical Question: Who assigns timestamps?**

#### Option A: User Provides Timestamps
```rust
subject.next_with_timestamp(value, timestamp);
```

**Pros:**
- ✅ Explicit control over event time
- ✅ Can integrate external events with known timestamps
- ✅ No assumptions about timestamp semantics

**Cons:**
- ❌ More verbose API
- ❌ User must manage timestamp generation
- ❌ Easy to make mistakes (non-monotonic timestamps)

**Use Case:** Integrating external events with inherent timestamps (logs, metrics, database events)

#### Option B: Auto-Assign on Push
```rust
subject.next(value); // Assigns timestamp internally
```

**Pros:**
- ✅ Ergonomic and simple
- ✅ Guarantees monotonic timestamps
- ✅ Matches reactive programming patterns (RxJS, etc.)

**Cons:**
- ❌ Timestamp = wall clock time (may not reflect event time)
- ❌ Needs strategy: `Instant::now()`, monotonic counter, etc.
- ❌ Less control for users

**Use Case:** Real-time UI events, user interactions, generated events

#### Option C: Hybrid Approach (Recommended)
```rust
subject.next(value);                          // Auto-timestamp
subject.next_with_timestamp(value, ts);       // Explicit timestamp
subject.next_sequenced(Sequenced::new(value)); // Pre-wrapped
```

**Pros:**
- ✅ Flexibility for different use cases
- ✅ Matches existing Fluxion patterns (`Sequenced::new()` vs `Sequenced::with_timestamp()`)
- ✅ Progressive disclosure (simple API with escape hatches)

**Cons:**
- ❌ More API surface
- ❌ Need clear documentation on when to use each

### 3. Broadcast Semantics

**How do multiple subscribers receive values?**

#### Option A: Clone-Based (like `tokio::broadcast`)
```rust
let subject = FluxionSubject::new();
let stream1 = subject.subscribe(); // FluxionStream
let stream2 = subject.subscribe(); // FluxionStream
subject.next(value); // Both streams receive it (cloned)
```

**Pros:**
- ✅ Each subscriber gets independent copy
- ✅ Familiar pattern (tokio::broadcast, RxJS)
- ✅ No lifetime issues

**Cons:**
- ❌ Requires `T: Clone`
- ❌ Performance cost for large values
- ❌ Memory overhead with many subscribers

#### Option B: Arc-Based (Shared Ownership)
```rust
subject.next(Arc::new(value)); // User manages Arc
// Or internal:
let subject = FluxionSubject::<Arc<T>>::new();
```

**Pros:**
- ✅ No clone requirement
- ✅ Efficient for large values
- ✅ Single allocation per value

**Cons:**
- ❌ Less ergonomic (Arc visible in API)
- ❌ Values are immutable
- ❌ Lifetime tracking complexity

#### Option C: Single-Consumer (Not Really a Subject)
```rust
let (subject, stream) = FluxionSubject::channel();
// Similar to current channel pattern
```

**Pros:**
- ✅ Simpler implementation
- ✅ No clone requirement
- ✅ Clear ownership

**Cons:**
- ❌ Loses multi-subscriber benefit
- ❌ Not really a "Subject" pattern
- ❌ Less useful for event buses, UI state

**Recommendation:** Option A (clone-based) for simplicity, with Option B available for performance-critical scenarios.

### 4. Error and Completion Handling

**Should Subject support error emission and completion signals?**

```rust
// Emit values
subject.next(value);

// Emit error
subject.error(FluxionError::new("something failed"));

// Signal completion
subject.complete(); // No more values
```

**Pros:**
- ✅ Matches Rx Observable pattern
- ✅ Clean lifecycle management
- ✅ Enables proper stream termination
- ✅ Composable with error handling operators

**Cons:**
- ❌ Added complexity
- ❌ Need to handle state (active vs completed vs errored)
- ❌ What happens if you call `.next()` after `.complete()`?

**Alternative:** Keep it simple, just values. Completion = drop Subject.

**Recommendation:** Start with error/completion support to match reactive patterns, but make it optional.

### 5. Backpressure Strategy

**What happens when subscribers can't keep up?**

#### Option A: Bounded + Blocking
```rust
let subject = FluxionSubject::with_capacity(100);
subject.next(value).await; // Blocks if buffer full
```

**Pros:**
- ✅ Backpressure propagates to source
- ✅ Memory bounded
- ✅ No data loss

**Cons:**
- ❌ Can deadlock if not careful
- ❌ Async API required
- ❌ Slower throughput

#### Option B: Bounded + Drop Policy
```rust
let subject = FluxionSubject::with_capacity(100)
    .drop_oldest(); // Or .drop_newest()
```

**Pros:**
- ✅ Never blocks
- ✅ Memory bounded
- ✅ Simple mental model

**Cons:**
- ❌ Data loss (dropped items)
- ❌ No backpressure signal
- ❌ Silent failures

**Like:** `tokio::broadcast` behavior

#### Option C: Unbounded
```rust
let subject = FluxionSubject::new(); // Unbounded by default
```

**Pros:**
- ✅ Simple, never blocks
- ✅ No data loss
- ✅ Fast

**Cons:**
- ❌ Memory can grow unbounded
- ❌ Risk of OOM in high-throughput scenarios
- ❌ No backpressure

**Recommendation:** Provide all three options, default to unbounded with clear documentation about risks.

### 6. Hot vs Cold Streams

#### Hot Subject (Standard)
- Values emitted to current subscribers only
- Late subscribers miss earlier values
- "Live broadcast"

```rust
let subject = FluxionSubject::new();
subject.next(1);
subject.next(2);
let stream = subject.subscribe(); // Misses 1 and 2
subject.next(3); // stream receives 3
```

#### Replay Subject
- Buffers last N values
- New subscribers receive buffered history
- "DVR mode"

```rust
let subject = ReplaySubject::new(5); // Buffer last 5 items
subject.next(1);
subject.next(2);
let stream = subject.subscribe(); // Immediately receives 1, 2
subject.next(3); // Receives 3
```

#### Behavior Subject
- Always has a "current value"
- New subscribers immediately get latest
- Useful for state management

```rust
let subject = BehaviorSubject::new(initial_value);
let stream = subject.subscribe(); // Immediately receives initial_value
subject.next(new_value);
```

**Recommendation:** Start with Hot Subject, add Replay/Behavior as separate types later.

## Potential API Design

### Basic Subject

```rust
// Create subject
let subject = FluxionSubject::<Sequenced<i32>>::new();

// Subscribe (get FluxionStream)
let stream1 = subject.subscribe();
let stream2 = subject.subscribe();

// Push values
subject.next(42); // Auto-assigns timestamp
subject.next_with_timestamp(100, timestamp);
subject.next_sequenced(Sequenced::new(42));

// Error/completion
subject.error(FluxionError::stream_error("failed"));
subject.complete();
```

### With Capacity

```rust
// Bounded with drop policy
let subject = FluxionSubject::with_capacity(100)
    .drop_oldest(); // Or .drop_newest()

// Bounded with backpressure
let subject = FluxionSubject::with_capacity(100)
    .blocking(); // .next() becomes async
```

### Replay Subject

```rust
let subject = ReplaySubject::new(5); // Buffer last 5 items
subject.next(1);
subject.next(2);
let stream = subject.subscribe(); // Gets 1, 2 immediately
```

### Behavior Subject

```rust
let subject = BehaviorSubject::new(initial_value);
let stream = subject.subscribe(); // Gets initial_value immediately
subject.next(new_value);
let current = subject.value(); // Get current value
```

## Implementation Considerations

### Internal Architecture

**Could use:**
- `tokio::sync::broadcast` for multi-subscriber broadcast
- Wrapper that assigns timestamps on `.next()`
- Maintains `FluxionStream` contract via `subscribe()` method
- Monotonic counter or `Instant::now()` for timestamp generation

**Example Structure:**
```rust
pub struct FluxionSubject<T> {
    sender: broadcast::Sender<StreamItem<T>>,
    timestamp_gen: Arc<Mutex<TimestampGenerator>>,
}

impl<T> FluxionSubject<T> {
    pub fn subscribe(&self) -> FluxionStream<impl Stream<Item = StreamItem<T>>> {
        let receiver = self.sender.subscribe();
        FluxionStream::from_broadcast_receiver(receiver)
    }

    pub fn next(&self, value: T::Inner) -> Result<()>
    where T: FluxionItem
    {
        let timestamp = self.timestamp_gen.lock().unwrap().next();
        let item = T::with_timestamp(value, timestamp);
        self.sender.send(StreamItem::Value(item))?;
        Ok(())
    }
}
```

### Challenges

1. **Ordering across pushes**: Need monotonic timestamp source
   - Solution: Internal counter or careful use of `Instant::now()`

2. **Thread safety**: Multiple threads pushing simultaneously
   - Solution: Internal `Mutex` or lock-free counter

3. **Subscriber management**: Dynamic add/remove
   - Solution: `broadcast::Sender` handles this automatically

4. **Memory management**: Preventing unbounded growth
   - Solution: Configurable capacity with drop policies

5. **Type safety**: Ensuring `T: FluxionItem`
   - Solution: Trait bounds on Subject creation

## Open Questions

1. **Do you need multi-subscriber?**
   - Likely YES for UI state, event buses
   - Could start single-consumer, add broadcast later

2. **Timestamp source?**
   - Wall clock (`Instant::now()`)
   - Monotonic counter (simpler, faster)
   - User-provided (hybrid approach)
   - **Recommendation:** Start with monotonic counter, add clock option later

3. **Error/completion semantics?**
   - YES for proper reactive patterns
   - Matches Rx Observable
   - Enables clean shutdown

4. **Backpressure strategy?**
   - Default: Unbounded (simple)
   - Optional: Bounded with drop policies
   - Advanced: Bounded with async blocking

5. **Replay capability?**
   - Start without (Hot Subject only)
   - Add ReplaySubject/BehaviorSubject as separate types later

6. **Should Subject be part of fluxion-stream or separate crate?**
   - Start in fluxion-stream for tight integration
   - Extract to fluxion-subject if it grows large

## Relation to Current Design

**Current Pattern:**
```rust
use fluxion_stream::IntoFluxionStream;

let (tx, rx) = futures::channel::mpsc::unbounded();
let stream = rx.into_fluxion_stream();
tx.send(Sequenced::new(value)).unwrap();
```

**With Subject:**
```rust
let subject = FluxionSubject::new();
let stream = subject.subscribe();
subject.next(value); // Auto-wrapped in Sequenced
```

**Subject essentially wraps the channel pattern with:**
- ✅ Easier API (no manual channel creation)
- ✅ Built-in timestamp assignment
- ✅ Broadcast support (multiple subscribers)
- ✅ Type-safe builder pattern
- ✅ Error/completion lifecycle

## Recommended Implementation Path

### Phase 1: MVP (Hot Subject)
- Single hot subject with clone-based broadcast
- Auto-timestamp with monotonic counter
- Unbounded by default
- Error/completion support
- Simple `subscribe()` API

### Phase 2: Capacity Management
- Add bounded variants
- Drop policies (oldest/newest)
- Optional async blocking

### Phase 3: Advanced Subjects
- ReplaySubject (buffer N items)
- BehaviorSubject (current value)
- Custom timestamp strategies

### Phase 4: Optimization
- Arc-based option for large values
- Lock-free implementations
- Performance benchmarks

## Next Steps

1. **API Surface Sketch**: Define exact method signatures
2. **Prototype**: Build MVP using `tokio::broadcast`
3. **Tests**: Verify multi-subscriber, ordering, error handling
4. **Documentation**: Usage examples, comparison to channels
5. **Integration**: How it composes with existing operators
