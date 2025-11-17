# Fluxion Stream Operators

A comprehensive guide to all stream operators available in `fluxion-stream`.

## Quick Reference Table

| Operator | Category | Purpose | Emissions Driven By |
|----------|----------|---------|---------------------|
| [`ordered_merge`](#ordered_merge) | Combining | Merge multiple streams temporally | All streams |
| [`combine_latest`](#combine_latest) | Combining | Combine latest from all streams | Any stream |
| [`with_latest_from`](#with_latest_from) | Combining | Sample secondary on primary | Primary only |
| [`combine_with_previous`](#combine_with_previous) | Windowing | Pair consecutive values | Source |
| [`map_ordered`](#map_ordered) | Transformation | Transform items | Source |
| [`filter_ordered`](#filter_ordered) | Filtering | Filter items | Source |
| [`take_while_with`](#take_while_with) | Filtering | Take while predicate holds | Source + Filter |
| [`take_latest_when`](#take_latest_when) | Sampling | Sample on trigger | Trigger |
| [`emit_when`](#emit_when) | Gating | Gate with combined state | Source (filtered) |

## Operators by Category

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

## Order Attribute Semantics

Every item in a Fluxion stream has an `order` attribute (accessed via `.order()`) that represents its temporal position in the event sequence. Understanding which order is preserved in emitted values is crucial for correct stream processing.

### Rules by Operator

| Operator | Order of Emitted Values | Rationale |
|----------|-------------------------|-----------|
| `ordered_merge` | Original source order | Pass-through operator |
| `map_ordered` | Original source order | Transformation preserves timing |
| `filter_ordered` | Original source order | Filtering preserves timing |
| `combine_with_previous` | Current value's order | Window driven by current item |
| `combine_latest` | Triggering stream's order | Event occurred when any stream updated |
| `with_latest_from` | **Primary stream's order** | Emission driven by primary |
| `take_latest_when` | **Trigger stream's order** | Emission driven by trigger |
| `emit_when` | **Source stream's order** | Source value emitted, not filter |
| `take_while_with` | Source stream's order | Source-driven with filter check |

### Event-Driven Semantics

For operators that combine multiple streams, the order represents **when the emission event occurred**, not when the underlying data was originally created.

#### ✅ Correct: `take_latest_when` and `with_latest_from`

These operators use the **triggering stream's order** because:

1. **Event-driven semantics**: The order represents when the emission *event* occurred
2. **Causal accuracy**: The trigger caused the emission at that moment
3. **Ordering guarantees**: Prevents violations in downstream operators
4. **Consistency**: Matches the "sampling" semantic model

**Example:**
```rust
// Sensor readings sampled by timer ticks
let sensor_stream = ...;  // order = when reading was taken (e.g., seq 1, 2, 3)
let timer_stream = ...;   // order = when tick occurred (e.g., seq 4, 5, 6)
let sampled = sensor_stream.take_latest_when(timer_stream, |_| true);
// First emission: sensor value from seq 3, but order = 4 (when tick occurred)
// This is correct - order represents "when we sampled", not "when data was created"
```

#### ✅ Also Correct: `emit_when` uses source order

While `emit_when` also involves a filter stream, it uses the **source stream's order** because:

1. **Source-driven**: The source value itself is being emitted (just conditionally)
2. **Filter is a gate**: The filter doesn't trigger emission, it just allows/blocks it
3. **Data identity**: The emitted value is the source value with its original timing

**Example:**
```rust
// Emit data only when it exceeds a threshold
let data_stream = ...;     // order = when data arrived
let threshold_stream = ...; // order = when threshold changed
let filtered = data_stream.emit_when(threshold_stream, |state| {
    state.values()[0] > state.values()[1]
});
// Emissions use data_stream's order - the data's original timestamp
```

### Why This Matters

**Downstream ordering**: Operators downstream expect monotonically increasing orders. Using the source's order when the trigger is newer could cause:
```rust
// ❌ HYPOTHETICAL PROBLEM (if we used source order in take_latest_when):
sensor.take_latest_when(timer, |_| true)  // Emits sensor order
    .combine_with_previous()  // Expects monotonic order
// Could emit: seq 10 (timer), then seq 3 (old sensor) → ordering violation!
```

**Correct behavior** (current implementation):
```rust
// ✅ CURRENT BEHAVIOR:
sensor.take_latest_when(timer, |_| true)  // Emits timer order
    .combine_with_previous()  // Receives monotonic order
// Emits: seq 10, seq 11, seq 12... → correct ordering!
```

### Time-Series Data Considerations

If you need to preserve the original timestamp of source data for provenance:

1. **Include timestamp in data**: Embed the original timestamp as part of your value type
2. **Use metadata**: Carry original timing information in your data structure
3. **Document semantics**: Clearly document that `order` represents emission time, not data time

```rust
// Pattern: Preserve original timestamp in data
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct TimestampedData {
    value: i32,
    original_timestamp: u64,  // Data's creation time
}

let sampled = stream.take_latest_when(timer, |_| true);
// Emission order = timer's order (when we sampled)
// But original_timestamp preserved in data (when data was created)
```

---

## See Also

- **[Integration Guide](../INTEGRATION.md)** - How to integrate events into Fluxion streams
- **[stream-aggregation example](../examples/stream-aggregation/)** - Production-ready patterns
- **[API Documentation](https://docs.rs/fluxion-rx)** - Complete API reference
- **[Operators Roadmap](FLUXION_OPERATORS_ROADMAP.md)** - Planned future operators
