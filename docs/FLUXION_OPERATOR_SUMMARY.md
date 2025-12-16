# Fluxion Stream Operators

A comprehensive guide to all stream operators available in `fluxion-stream`.

**Note:** For time-based operators (`delay`, `debounce`, `throttle`, `sample`, `timeout`), see the **[fluxion-stream-time](../fluxion-stream-time/README.md)** crate documentation.

## Quick Reference Table

| Operator | Category | Purpose | Emissions Driven By |
|----------|----------|---------|---------------------|
| [`ordered_merge`](#ordered_merge) | Combining | Merge multiple streams temporally | All streams |
| [`merge_with`](#merge_with) | Combining | Stateful merging with shared state | All streams |
| [`combine_latest`](#combine_latest) | Combining | Combine latest from all streams | Any stream |
| [`with_latest_from`](#with_latest_from) | Combining | Sample secondary on primary | Primary only |
| [`start_with`](#start_with) | Combining | Prepend initial values | Source |
| [`combine_with_previous`](#combine_with_previous) | Windowing | Pair consecutive values | Source |
| [`scan_ordered`](#scan_ordered) | Transformation | Accumulate state, emit intermediate results | Source |
| [`map_ordered`](#map_ordered) | Transformation | Transform items | Source |
| [`filter_ordered`](#filter_ordered) | Filtering | Filter items | Source |
| [`distinct_until_changed`](#distinct_until_changed) | Filtering | Suppress consecutive duplicates | Source |
| [`distinct_until_changed_by`](#distinct_until_changed_by) | Filtering | Custom duplicate suppression | Source |
| [`take_while_with`](#take_while_with) | Filtering | Take while predicate holds | Source + Filter |
| [`take_items`](#take_items) | Limiting | Take first N items | Source |
| [`skip_items`](#skip_items) | Limiting | Skip first N items | Source |
| [`take_latest_when`](#take_latest_when) | Sampling | Sample on trigger | Trigger |
| [`sample_ratio`](#sample_ratio) | Sampling | Probabilistic downsampling | Source |
| [`emit_when`](#emit_when) | Gating | Gate with combined state | Source (filtered) |
| [`partition`](#partition) | Splitting | Split stream into two by predicate | Source |
| [`on_error`](#on_error) | Error Handling | Selectively consume or propagate errors | Source |
| [`share`](#share) | Multicasting | Broadcast to multiple subscribers | Source |
| [`subscribe`](#subscribe) ⚡ | Execution | Process every item sequentially | Source |
| [`subscribe_latest`](#subscribe_latest) ⚡ | Execution | Process latest item, cancel outdated | Source |
| `debounce` ⏱️ | Time | Emit after silence | Source (debounced) |
| `throttle` ⏱️ | Time | Rate limiting | Source (throttled) |
| `delay` ⏱️ | Time | Delay emissions | Source (delayed) |
| `sample` ⏱️ | Time | Periodic sampling | Time intervals |
| `timeout` ⏱️ | Time | Timeout detection | Source or timeout |

**⏱️** = Available in [fluxion-stream-time](../fluxion-stream-time/README.md) crate
**⚡** = Available in [fluxion-exec](../fluxion-exec/README.md) crate

## Operators by Category

### 🔀 Combining Streams
- [`ordered_merge`](#ordered_merge) - Merge streams in temporal order
- [`merge_with`](#merge_with) - Stateful merging with shared state
- [`combine_latest`](#combine_latest) - Combine latest from all streams
- [`with_latest_from`](#with_latest_from) - Sample secondary on primary emission
- [`start_with`](#start_with) - Prepend initial values to stream

### 🪟 Windowing & Pairing
- [`combine_with_previous`](#combine_with_previous) - Pair consecutive values

### 🔄 Transformation
- [`scan_ordered`](#scan_ordered) - Accumulate state, emit intermediate results
- [`map_ordered`](#map_ordered) - Transform items

### 🔍 Filtering
- [`filter_ordered`](#filter_ordered) - Filter items by predicate
- [`take_items`](#take_items) - Take first N items
- [`skip_items`](#skip_items) - Skip first N items
- [`distinct_until_changed`](#distinct_until_changed) - Suppress consecutive duplicates
- [`distinct_until_changed_by`](#distinct_until_changed_by) - Custom duplicate suppression
- [`take_while_with`](#take_while_with) - Take while condition holds

### 📊 Sampling & Gating
- [`take_latest_when`](#take_latest_when) - Sample on trigger events
- [`sample_ratio`](#sample_ratio) - Probabilistic downsampling by ratio
- [`emit_when`](#emit_when) - Gate based on combined state

### ✂️ Splitting
- [`partition`](#partition) - Split stream into two based on predicate

### 🛡️ Error Handling
- [`on_error`](#on_error) - Selectively consume or propagate errors

### 📡 Multicasting
- [`share`](#share) - Broadcast stream to multiple subscribers

### ⚡ Execution (fluxion-exec)
- [`subscribe`](#subscribe) - Sequential processing of all items
- [`subscribe_latest`](#subscribe_latest) - Latest-value processing with cancellation

### ⏱️ Time-Based Operators (fluxion-stream-time)
- `debounce` - Emit only after silence period
- `throttle` - Emit at most once per time window
- `delay` - Delay all emissions by duration
- `sample` - Sample at regular intervals
- `timeout` - Emit error if no items within duration

**See [fluxion-stream-time documentation](../fluxion-stream-time/README.md) for details**

---

### 🔀 Combining Streams

#### `ordered_merge`
**Merge multiple streams preserving temporal order**

```rust
let merged = stream1.ordered_merge(vec![stream2, stream3]);
```

- Emits items from all streams in temporal sequence order
- No transformation—items pass through as-is
- Foundation for multi-source event aggregation
- See API docs for detailed examples

---

#### `merge_with`
**Stateful merging of multiple streams with shared state**

```rust
let merged = MergedStream::seed::<Sequenced<Event>>(Repository::new())
    .merge_with(user_stream, |event, repo| {
        repo.users.insert(event.user_id, event.user);
        Event::UserAdded(event.user_id)
    })
    .merge_with(order_stream, |event, repo| {
        repo.orders.insert(event.order_id, event.order);
        Event::OrderCreated(event.order_id)
    })
    .into_fluxion_stream();
```

- Maintains shared mutable state across all merged streams
- Each processing function has mutable access to state
- Processes events in temporal order (uses `ordered_merge` internally)
- Chain multiple `merge_with` calls for complex state management
- Convert to `FluxionStream` via `into_fluxion_stream()` for operator chaining
- Essential for repository pattern and event sourcing
- See API docs for detailed examples

---

#### `combine_latest`
**Combine latest values from multiple streams**

```rust
let combined = stream1.combine_latest(vec![stream2], |state| {
    // Filter predicate
    true
});
```

- Emits `CombinedState<T>` when **any** stream emits
- Contains latest value from all streams
- Useful for multi-source aggregation
- See API docs for detailed examples

---

#### `with_latest_from`
**Sample secondary streams on primary emission**

```rust
let combined = primary.with_latest_from(secondary, |state| {
    // Transform combined state
    state.values()[0] + state.values()[1]
});
```

- Emissions occur **only when primary emits**
- Samples latest from secondary stream(s)
- Primary-driven combination pattern
- See API docs for detailed examples

---

#### `start_with`
**Prepend initial values to stream**

```rust
let with_defaults = stream.start_with(vec![
    StreamItem::Value(Sequenced::new(default_value1)),
    StreamItem::Value(Sequenced::new(default_value2)),
]);
```

- Emits initial values before any source stream values
- Useful for providing default/placeholder values
- Initial values can include errors for testing error handling
- See API docs for detailed examples

---

### 🪟 Windowing & Pairing

#### `combine_with_previous`
**Create sliding window of consecutive values**

```rust
let paired = stream.combine_with_previous();
// Emits: WithPrevious { previous: Option<T>, current: T }
```

- First emission has `previous = None`
- Subsequent emissions pair consecutive values
- Useful for delta calculation and change detection
- See API docs for detailed examples

---

### 🔄 Transformation

#### `scan_ordered`
**Accumulate state across stream items, emitting intermediate results**

```rust
// Running sum
let sums = stream.scan_ordered::<Sequenced<i32>, _, _>(0, |acc, val| {
    *acc += val;
    *acc
});

// State machine
let states = events.scan_ordered::<Sequenced<State>, _, _>(
    State::initial(),
    |state, event| {
        state.transition(event);
        state.clone()
    }
);
```

- Maintains accumulator state across all stream items
- Emits transformed value for each input item
- Can transform types (e.g., i32 → String, Event → State)
- Errors propagate without resetting state
- Useful for running totals, state machines, building collections
- See API docs for detailed examples

---

#### `map_ordered`
**Transform items while preserving temporal order**

```rust
let transformed = stream.map_ordered(|item| {
    format!("Value: {}", item.get())
});
```

- Maintains ordering guarantees (unlike `StreamExt::map`)
- Preserves `FluxionStream` wrapper
- Essential for operator chaining
- See API docs for detailed examples

---

### 🔍 Filtering

#### `filter_ordered`
**Filter items while preserving temporal order**

```rust
let filtered = stream.filter_ordered(|value| value % 2 == 0);
```

- Maintains ordering guarantees (unlike `StreamExt::filter`)
- Preserves `FluxionStream` wrapper
- Filters based on source value only
- See API docs for detailed examples

---

#### `take_items`
**Emit only the first N items**

```rust
let limited = stream.take_items(10);
```

- Emits at most N items then completes
- Errors count as items (use `on_error()` first to filter errors)
- Stream completes after N items
- Useful for pagination, testing, and limiting results
- See API docs for detailed examples

---

#### `skip_items`
**Skip the first N items**

```rust
let after_skip = stream.skip_items(5);
```

- Discards first N items, emits all remaining items
- Errors count as items (use `on_error()` first to filter errors)
- Useful for pagination and skipping initial values
- See API docs for detailed examples

---

#### `distinct_until_changed`
**Suppress consecutive duplicate values**

```rust
let distinct = stream.distinct_until_changed();
```

- Filters out consecutive duplicate values using `PartialEq`
- Only compares adjacent items (not global deduplication)
- Maintains temporal ordering
- Useful for change detection and noise reduction
- See API docs for detailed examples

---

#### `distinct_until_changed_by`
**Custom duplicate suppression with comparison function**

```rust
// Field-based comparison
let distinct = stream.distinct_until_changed_by(|a, b| a.id == b.id);

// Case-insensitive string comparison
let distinct = stream.distinct_until_changed_by(|a, b| {
    a.to_lowercase() == b.to_lowercase()
});

// Threshold-based comparison
let distinct = stream.distinct_until_changed_by(|a, b| (a - b).abs() < 0.5);
```

- Custom comparison function for flexible duplicate detection
- No `PartialEq` requirement on inner type
- Comparison function returns `true` if values considered equal (filtered)
- Useful for field comparison, case-insensitive matching, threshold filtering
- Follows Rust patterns: `sort_by`, `dedup_by`, `max_by`
- See API docs for detailed examples

---

#### `take_while_with`
**Emit while external condition holds**

```rust
let taken = stream.take_while_with(condition_stream, |cond| *cond > 0);
```

- Emits source items while filter stream's value satisfies predicate
- Stream completes when predicate returns false
- Conditional flow control
- See API docs for detailed examples

---

### 📊 Sampling & Gating

#### `take_latest_when`
**Sample latest value on trigger events**

```rust
let sampled = stream.take_latest_when(trigger, |_| true);
```

- Buffers source values until trigger emits
- Emits latest buffered value on trigger
- After first trigger, source emits immediately
- Rate limiting and event-driven sampling
- See API docs for detailed examples

---

#### `sample_ratio`
**Probabilistic downsampling by ratio**

**Signature:**
```rust
fn sample_ratio(self, ratio: f64, seed: u64) -> impl Stream<Item = StreamItem<T>>
```

**Basic Usage:**
```rust
use fluxion_stream::prelude::*;
use fluxion_test_utils::{Sequenced, test_channel};
use futures::StreamExt;

let (tx, stream) = test_channel::<Sequenced<i32>>();

// Sample approximately 50% of items
let sampled = stream.sample_ratio(0.5, 42);

// In production, use random seed:
// let sampled = stream.sample_ratio(0.1, fastrand::u64(..));
```

**Behavior:**
- **Ratio range** - `0.0` (emit nothing) to `1.0` (emit all), panics if outside range
- **Deterministic** - Same seed produces same sampling pattern for testing
- **Error pass-through** - Errors always pass through, never subject to sampling
- **Timestamp-preserving** - Original timestamps maintained on sampled items
- **Stateless per-item** - Each item evaluated independently (no windowing)

**Use Cases:**
- **Load reduction** - Reduce downstream processing load on high-frequency streams
- **Logging sampling** - Log a representative sample of events
- **Monitoring** - Downsample metrics while preserving error visibility
- **Testing** - Deterministic seed enables reproducible sampling behavior

**Production vs Testing:**
```rust
// Testing - deterministic sampling with fixed seed
let sampled = stream.sample_ratio(0.5, 42);

// Production - random sampling
let sampled = stream.sample_ratio(0.1, fastrand::u64(..));
```

**See Also:**
- [`take_latest_when`](#take_latest_when) - For trigger-based sampling
- `sample` (⏱️ fluxion-stream-time) - For time-based sampling

[Full documentation](../fluxion-stream/src/sample_ratio.rs) | [Tests](../fluxion-stream/tests/sample_ratio/) | [Benchmarks](../fluxion-stream/benches/sample_ratio_bench.rs)

---

#### `emit_when`
**Gate emissions based on combined state**

```rust
let gated = source.emit_when(threshold, |state| {
    state.values()[0] > state.values()[1]
});
```

- Combines source and filter into `CombinedState`
- Predicate evaluates both values
- Emits source only when predicate is true
- Dynamic threshold filtering
- See API docs for detailed examples

---

## Common Patterns

### Pattern 1: Multi-Stream Aggregation
```rust
// Merge multiple sources, pair consecutive values, transform
let pipeline = stream1
    .ordered_merge(vec![stream2, stream3])
    .combine_with_previous()
    .map_ordered(|paired| compute_delta(&paired));
```

### Pattern 2: Threshold-Based Processing
```rust
// Emit values only when they exceed a dynamic threshold
let filtered = data_stream
    .emit_when(threshold_stream, |state| {
        state.values()[0] > state.values()[1]
    });
```

### Pattern 3: Sampling with Context
```rust
// Combine data with configuration, filter, transform
let enriched = data_stream
    .with_latest_from(config_stream, |state| {
        // Combine data with config
        (state.values()[0].clone(), state.values()[1].clone())
    })
    .filter_ordered(|(data, config)| validate(data, config));
```

### Pattern 4: Rate Limiting
```rust
// Sample a high-frequency stream at a lower rate
let sampled = fast_stream
    .take_latest_when(slow_trigger, |_| true);
```

---

## Choosing the Right Operator

### When to use `ordered_merge` vs `combine_latest`
- **`ordered_merge`**: Items pass through individually in temporal order
- **`combine_latest`**: Items combined into `CombinedState`, emitted on any change

### When to use `with_latest_from` vs `combine_latest`
- **`with_latest_from`**: Primary-driven, samples secondary
- **`combine_latest`**: All streams drive emissions equally

### When to use `take_latest_when` vs `emit_when`
- **`take_latest_when`**: Simple trigger sampling, filter checks trigger value
- **`emit_when`**: Complex filtering, predicate evaluates combined state

### When to use `take_while_with` vs `filter_ordered`
- **`take_while_with`**: Conditional flow control with external stream, terminates on false
- **`filter_ordered`**: Simple value-based filtering, never terminates

---

## Type Requirements

All operators require items to implement:
- `Ordered` - For sequence number access
- `Clone + Debug + Ord` - For comparison and debugging
- `Send + Sync + Unpin + 'static` - For async streaming

The `Sequenced<T>` wrapper (from `fluxion-test-utils`) provides this automatically.

---

## Timestamp Semantics

Every item in a Fluxion stream has a `timestamp` attribute (accessed via `.timestamp()`) that represents its temporal position in the event sequence. Understanding which timestamp is preserved in emitted values is crucial for correct stream processing.

### Rules by Operator

| Operator | Timestamp of Emitted Values | Rationale |
|----------|-------------------------|-----------|
| `ordered_merge` | Original source timestamp | Pass-through operator |
| `merge_with` | Original source timestamp | Stateful transformation preserves source timing |
| `scan_ordered` | Original source timestamp | Stateful transformation preserves source timing |
| `map_ordered` | Original source timestamp | Transformation preserves timing |
| `filter_ordered` | Original source timestamp | Filtering preserves timing |
| `distinct_until_changed` | Original source timestamp | Filtering preserves timing |
| `distinct_until_changed_by` | Original source timestamp | Filtering preserves timing |
| `take_items` | Original source timestamp | Limiting preserves timing |
| `skip_items` | Original source timestamp | Limiting preserves timing |
| `on_error` | Original source timestamp | Error handling preserves timing |
| `start_with` | User-provided timestamp | Initial values have their own timestamps |
| `combine_with_previous` | Current value's timestamp | Window driven by current item |
| `combine_latest` | Triggering stream's timestamp | Event occurred when any stream updated |
| `with_latest_from` | **Primary stream's timestamp** | Emission driven by primary |
| `take_latest_when` | **Trigger stream's timestamp** | Emission driven by trigger |
| `emit_when` | **Source stream's timestamp** | Source value emitted, not filter |
| `take_while_with` | Source stream's timestamp | Source-driven with filter check |

### Event-Driven Semantics

For operators that combine multiple streams, the timestamp represents **when the emission event occurred**, not when the underlying data was originally created.

#### ✅ Correct: `take_latest_when` and `with_latest_from`

These operators use the **triggering stream's timestamp** because:

1. **Event-driven semantics**: The timestamp represents when the emission *event* occurred
2. **Causal accuracy**: The trigger caused the emission at that moment
3. **Ordering guarantees**: Prevents violations in downstream operators
4. **Consistency**: Matches the "sampling" semantic model

**Example:**
```rust
// Sensor readings sampled by timer ticks
let sensor_stream = ...;  // timestamp = when reading was taken (e.g., seq 1, 2, 3)
let timer_stream = ...;   // timestamp = when tick occurred (e.g., seq 4, 5, 6)
let sampled = sensor_stream.take_latest_when(timer_stream, |_| true);
// First emission: sensor value from seq 3, but timestamp = 4 (when tick occurred)
// This is correct - timestamp represents "when we sampled", not "when data was created"
```

#### ✅ Also Correct: `emit_when` uses source timestamp

While `emit_when` also involves a filter stream, it uses the **source stream's timestamp** because:

1. **Source-driven**: The source value itself is being emitted (just conditionally)
2. **Filter is a gate**: The filter doesn't trigger emission, it just allows/blocks it
3. **Data identity**: The emitted value is the source value with its original timing

**Example:**
```rust
// Emit data only when it exceeds a threshold
let data_stream = ...;     // timestamp = when data arrived
let threshold_stream = ...; // timestamp = when threshold changed
let filtered = data_stream.emit_when(threshold_stream, |state| {
    state.values()[0] > state.values()[1]
});
// Emissions use data_stream's timestamp - the data's original timestamp
```

### Why This Matters

**Downstream ordering**: Operators downstream expect monotonically increasing timestamps. Using the source's timestamp when the trigger is newer could cause:
```rust
// ❌ HYPOTHETICAL PROBLEM (if we used source timestamp in take_latest_when):
sensor.take_latest_when(timer, |_| true)  // Emits sensor timestamp
    .combine_with_previous()  // Expects monotonic timestamp
// Could emit: seq 10 (timer), then seq 3 (old sensor) → ordering violation!
```

**Correct behavior** (current implementation):
```rust
// ✅ CURRENT BEHAVIOR:
sensor.take_latest_when(timer, |_| true)  // Emits timer timestamp
    .combine_with_previous()  // Receives monotonic timestamp
// Emits: seq 10, seq 11, seq 12... → correct ordering!
```

### Time-Series Data Considerations

If you need to preserve the original timestamp of source data for provenance:

1. **Include timestamp in data**: Embed the original timestamp as part of your value type
2. **Use metadata**: Carry original timing information in your data structure
3. **Document semantics**: Clearly document that `timestamp` represents emission time, not data time

```rust
// Pattern: Preserve original timestamp in data
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct TimestampedData {
    value: i32,
    original_timestamp: u64,  // Data's creation time
}

let sampled = stream.take_latest_when(timer, |_| true);
// Emission timestamp = timer's timestamp (when we sampled)
// But original_timestamp preserved in data (when data was created)
```

---

### 🛡️ Error Handling

#### `on_error`
**Selectively consume or propagate errors using Chain of Responsibility pattern**

**Signature:**
```rust
fn on_error<F>(self, handler: F) -> OnError<Self, F>
where
    F: FnMut(&FluxionError) -> bool;
```

**Basic Usage:**
```rust
use fluxion_rx::FluxionStream;

stream
    .on_error(|err| {
        log::error!("Stream error occurred: {}", err);
        true // Consume all errors, continue stream
    })
```

**Chain of Responsibility Pattern:**
```rust
let stream = stream
    // Handle network errors specifically
    .on_error(|err| {
        if matches!(err, FluxionError::Stream(msg) if msg.contains("network")) {
            log::warn!("Network error (retrying later): {}", err);
            metrics::increment("network_errors");
            true // Consume network errors
        } else {
            false // Not a network error, propagate
        }
    })
    // Handle validation errors
    .on_error(|err| {
        if matches!(err, FluxionError::User(_)) {
            log::info!("Validation error: {}", err);
            metrics::increment("validation_errors");
            true // Consume validation errors
        } else {
            false // Propagate
        }
    })
    // Catch-all for remaining errors
    .on_error(|err| {
        log::error!("Unhandled error: {}", err);
        metrics::increment("unhandled_errors");
        true // Consume all remaining errors
    })
```

**Error Type Discrimination:**
```rust
stream
    .on_error(|err| {
        match err {
            FluxionError::User(user_err) => {
                notify_user(&user_err);
                true
            }
            FluxionError::Stream(msg) if msg.contains("timeout") => {
                retry_later();
                true
            }
            FluxionError::MultipleErrors(_) => {
                alert_ops_team(err);
                true
            }
            _ => false // Propagate other errors
        }
    })
```

**Conditional Error Suppression:**
```rust
stream
    .on_error(|err| {
        // Only suppress errors during grace period
        if in_grace_period() {
            log::debug!("Suppressed error during grace period: {}", err);
            true
        } else {
            false // Propagate errors after grace period
        }
    })
```

**Behavior:**
- **When handler returns `true`** (error matched and handled):
  - Consume the error - Remove the `StreamItem::Error` from the stream
  - Continue processing - Allow subsequent `StreamItem::Value` items to flow normally
  - Enable side effects - Handler can log, send metrics, or perform other actions

- **When handler returns `false`** (error not matched):
  - Propagate the error - Pass `StreamItem::Error` downstream unchanged
  - Chain to next handler - Allow subsequent `on_error` operators to handle it

**Design Rationale:**
- **Chain of Responsibility Pattern** - Like try/catch blocks, handle specific errors first, then general
- **Composable** - Each handler is independent and focused on one error type
- **Explicit propagation** - Clear when errors are consumed vs propagated
- **Simple boolean return** - `true` = handled, `false` = propagate
- **Non-consuming** - Handler receives `&FluxionError`, doesn't own it (no cloning required)

**Error Handling Guarantees:**
- ✓ Stream continues after error consumption
- ✓ Subsequent values flow normally
- ✓ Multiple errors can be handled independently
- ✓ Unhandled errors propagate downstream
- ✓ Error order preserved
- ✓ No errors silently dropped (unless explicitly consumed)

**Performance Characteristics:**
- **Minimal overhead** - Single function call per error
- **No allocations** - Handler receives reference
- **Zero cost when no errors** - Only affects error path
- Handler inlining for simple predicates

**Comparison with Other Approaches:**
- vs. `catch` (Returns Replacement Value) - Fluxion just consumes, no replacement needed
- vs. `on_error_resume_next` (Returns Stream) - Simpler for most cases
- vs. `retry` (Automatic Retry) - Manual control for custom retry logic

**Common Patterns:**
- Logging and metrics collection
- Selective error filtering by type
- Conditional error recovery during grace periods
- Alert/notification triggering
- Side effect execution before propagation decision

**See Also:**
- [Error Handling Guide](ERROR-HANDLING.md) for comprehensive patterns
- [FluxionError types](../fluxion-core/src/error.rs) for error enum details

[Full documentation](../fluxion-stream/src/fluxion_stream.rs#L780-L866) | [Tests](../fluxion-stream/tests/on_error_tests.rs)

---

### ✂️ Splitting

#### `partition`
**Split a stream into two based on a predicate**

**Signature:**
```rust
fn partition<F>(self, predicate: F) -> (PartitionedStream<T>, PartitionedStream<T>)
where
    F: Fn(&T::Inner) -> bool + Send + Sync + 'static;
```

**Basic Usage:**
```rust
use fluxion_stream::{IntoFluxionStream, PartitionExt};
use fluxion_test_utils::Sequenced;
use futures::StreamExt;

let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

// Partition numbers into even and odd
let (mut evens, mut odds) = rx.into_fluxion_stream()
    .partition(|n: &i32| n % 2 == 0);

tx.send(Sequenced::new(1)).unwrap();
tx.send(Sequenced::new(2)).unwrap();
tx.send(Sequenced::new(3)).unwrap();
tx.send(Sequenced::new(4)).unwrap();
drop(tx);

// Consume both streams independently
// evens: 2, 4
// odds: 1, 3
```

**Error Routing Example:**
```rust
// Separate successful values from validation failures
let (valid, invalid) = events.partition(|event| event.is_valid());

// Process valid events normally
tokio::spawn(async move {
    while let Some(item) = valid.next().await {
        handle_valid(item.unwrap()).await;
    }
});

// Route invalid events to error queue
tokio::spawn(async move {
    while let Some(item) = invalid.next().await {
        send_to_dlq(item.unwrap()).await;
    }
});
```

**Behavior:**
- **Chain-breaking** - Returns two streams, cannot chain further on original
- **Spawns task** - Routing runs in a background Tokio task
- **Timestamp-preserving** - Original timestamps are preserved in both output streams
- **Routing** - Every item goes to exactly one output stream
- **Non-blocking** - Both streams can be consumed independently
- **Hot** - Uses internal subjects for broadcasting (late consumers miss items)
- **Error propagation** - Errors are sent to both output streams
- **Unbounded buffers** - Items are buffered in memory until consumed

**Buffer Behavior:**

The partition operator uses unbounded internal channels. If one partition stream is consumed slowly (or not at all), items destined for that stream will accumulate in memory:

- If you only consume one partition, items for the other still buffer
- For high-throughput streams with imbalanced consumption, consider adding backpressure mechanisms downstream
- Dropping one partition stream is safe; items for it are simply discarded

**Use Cases:**
- **Error routing** - Separate successful values from validation failures
- **Priority queues** - Split high-priority and low-priority items
- **Type routing** - Route different enum variants to specialized handlers
- **Threshold filtering** - Split values above/below a threshold

**Performance Characteristics:**
- Background task spawned immediately on partition creation
- Single pass through source stream
- Minimal overhead per item (predicate evaluation + channel send)
- Benchmarked for balanced (50/50) and imbalanced (90/10) splits

**See Also:**
- [`filter_ordered`](#filter_ordered) - For single-stream filtering (keeps items passing predicate)
- [`share`](#share) - For broadcasting same items to multiple consumers

[Full documentation](../fluxion-stream/src/partition.rs) | [Tests](../fluxion-stream/tests/partition_tests.rs) | [Benchmarks](../benchmarks/benches/partition_bench.rs)

---

### 📡 Multicasting

#### `share`
**Convert a cold stream into a hot, multi-subscriber broadcast source**

**Signature:**
```rust
fn share(self) -> FluxionShared<T>
where
    T: Clone + Send + Sync + 'static;
```

**Basic Usage:**
```rust
use fluxion_stream::{IntoFluxionStream, ShareExt, MapOrderedExt};
use fluxion_test_utils::Sequenced;

let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

// Create and share a source stream
let shared = rx.into_fluxion_stream()
    .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner() * 2))
    .share();

// Multiple subscribers receive the same broadcast
let sub1 = shared.subscribe().unwrap();
let sub2 = shared.subscribe().unwrap();

// Send value - both subscribers receive it
tx.send(Sequenced::new(5)).unwrap();
```

**Independent Subscriber Chains:**
```rust
let shared = source_stream.share();

// Each subscriber can chain operators independently
let evens = shared.subscribe().unwrap()
    .filter_ordered(|x| x.into_inner() % 2 == 0);

let strings = shared.subscribe().unwrap()
    .map_ordered(|x: Sequenced<i32>| Sequenced::new(x.into_inner().to_string()));
```

**Behavior:**
- **Hot stream** - Late subscribers do not receive past items
- **Shared execution** - Source stream consumed once, results broadcast to all
- **Subscription factory** - Call `subscribe()` to create independent subscriber streams
- **Owned lifecycle** - Forwarding task cancelled when `FluxionShared` is dropped
- **Error propagation** - Errors broadcast to all subscribers, then source closes

**Comparison with FluxionSubject:**

| Type | Source | Push API | Use Case |
|------|--------|----------|----------|
| `FluxionSubject` | External (you call `next()`) | Yes | Manual event emission |
| `FluxionShared` | Existing stream | No | Share computed stream |

**Common Patterns:**
- Share expensive computations across multiple consumers
- Fan-out pattern for event distribution
- Broadcast transformed data to multiple processors
- Decouple stream production from consumption

**Performance Characteristics:**
- Single source consumption regardless of subscriber count
- Broadcast overhead proportional to subscriber count
- Internal `FluxionSubject` handles subscriber management

**See Also:**
- [`FluxionSubject`](../fluxion-core/src/fluxion_subject.rs) for manual event emission
- [Error Handling Guide](ERROR-HANDLING.md) for error propagation patterns

[Full documentation](../fluxion-stream/src/fluxion_shared.rs) | [Tests](../fluxion-stream/tests/fluxion_shared/)

---

### ⚡ Execution (fluxion-exec)

These operators are available in the `fluxion-exec` crate and provide control over async execution.

#### `subscribe`
**Sequential processing where every item is processed to completion**

```rust
use fluxion_exec::subscribe::SubscribeExt;

stream.subscribe(
    |item, _| async move {
        process(item).await
    },
    None,
    None
).await?;
```

- Guarantees every item is processed
- Sequential execution (one at a time)
- See [fluxion-exec documentation](../fluxion-exec/README.md) for details

---

#### `subscribe_latest`
**Latest-value processing with automatic cancellation**

```rust
use fluxion_exec::subscribe_latest::SubscribeLatestExt;

stream.subscribe_latest(
    |item, token| async move {
        // Long running task
    },
    None,
    None
).await?;
```

- Processes only the latest value
- Automatically cancels outdated work
- Ideal for UI updates and search-as-you-type
- See [fluxion-exec documentation](../fluxion-exec/README.md) for details

---

## See Also

- **[Integration Guide](../INTEGRATION.md)** - How to integrate events into Fluxion streams
- **[stream-aggregation example](../examples/stream-aggregation/)** - Production-ready patterns
- **[API Documentation](https://docs.rs/fluxion-rx)** - Complete API reference
- **[Error Handling Guide](ERROR-HANDLING.md)** - Comprehensive error handling patterns
- **[Operators Roadmap](FLUXION_OPERATORS_ROADMAP.md)** - Planned future operators
