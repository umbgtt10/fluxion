# ADR 001: Ordered Merge Temporal Ordering Semantics

## Status

Accepted

## Context

The `ordered_merge` operator is a fundamental building block in Fluxion, responsible for combining multiple asynchronous streams while preserving temporal correctness. During development and testing, questions arose about the precise semantics of how items and errors flow through the operator, particularly:

1. **Why does an error appear at position 3 instead of position 4** in `test_ordered_merge_error_ordering_by_timestamp`?
2. **When are items emitted** - immediately upon arrival or after temporal reordering?
3. **How do errors interact with the temporal ordering buffer** - are they buffered or bypass?
4. **What are the implications for testing** - should tests use one-by-one or batch send/assert patterns?

Understanding these semantics is critical because:
- `ordered_merge` is used as the foundation for many other operators
- Incorrect mental models lead to flaky tests and subtle bugs
- The operator's behavior differs fundamentally from standard stream combinators like `futures::stream::select`

## Decision

### Three-Layer Architecture

`ordered_merge` implements a **three-layer architecture** for processing items:

#### Layer 1: Per-Stream FIFO Queues (Channel Semantics)
- Each input stream has its own FIFO queue (channel)
- Items arrive in the order they are sent to each individual stream
- **Guarantee**: Within a single stream, arrival order = emission order from that stream's channel

#### Layer 2: Cross-Stream Temporal Ordering Buffer
- **Values only** are buffered and sorted by timestamp
- Items are emitted when the operator can guarantee no earlier-timestamped value will arrive
- **Watermark mechanism**: The operator tracks the minimum timestamp across all active streams
- Items with timestamps ≤ watermark are safe to emit (no future item can be earlier)

#### Layer 3: Error Bypass Path
- **Errors bypass the temporal buffer entirely**
- Errors are emitted immediately upon arrival (FIFO from their source stream)
- **Watermark advancement**: Each error acts as an implicit watermark, advancing the minimum timestamp
- This allows buffered values to be released after an error

### Emission Order Example

Given this send sequence:
```rust
tx1.unbounded_send(Value(ts=1))?;  // Buffered
tx2.unbounded_send(Value(ts=2))?;  // Buffered (waiting for confirmation)
tx1.unbounded_send(Value(ts=3))?;  // Buffered
tx2.unbounded_send(Error)?;        // Emitted immediately (position 1)
tx1.unbounded_send(Value(ts=4))?;  // Buffered
```

**Emission order:**
1. **Error** (position 1) - bypasses buffer, emitted immediately
2. **Value(ts=1)** (position 2) - watermark advanced by error, safe to emit
3. **Value(ts=2)** (position 3) - next in temporal order
4. **Value(ts=3)** (position 4) - next in temporal order
5. **Value(ts=4)** (position 5) - emitted when subsequent value/error advances watermark

**Key Insight**: The error appears at position 1 (not position 4) because errors bypass the temporal ordering buffer.

### Error Semantics

1. **Immediate Emission**: Errors are never buffered - they flow through FIFO from their source stream
2. **Watermark Advancement**: Each error advances the watermark, allowing buffered values to be released
3. **No Reordering**: Multiple errors from different streams emit in the order they arrive at the operator's polling logic
4. **Stream Continuation**: Errors do not terminate streams - processing continues for subsequent items

### Testing Implications

#### ✅ Correct: Batch Send, Batch Assert
```rust
// Act
tx1.unbounded_send(Value(ts=2))?;
tx2.unbounded_send(Value(ts=1))?;
tx1.unbounded_send(Value(ts=3))?;

// Assert
assert_eq!(next(), Value(ts=1));  // Temporal reordering visible
assert_eq!(next(), Value(ts=2));
assert_eq!(next(), Value(ts=3));
```

This pattern tests the actual temporal ordering behavior - items sent out-of-order are emitted in timestamp order.

#### ❌ Incorrect: One-by-One Send/Assert
```rust
// Act & Assert mixed
tx1.unbounded_send(Value(ts=1))?;
assert_eq!(next(), Value(ts=1));  // Passes but...

tx2.unbounded_send(Value(ts=2))?;
assert_eq!(next(), Value(ts=2));  // ...doesn't test reordering

tx1.unbounded_send(Value(ts=3))?;
assert_eq!(next(), Value(ts=3));  // Temporal ordering never exercised
```

This pattern doesn't test temporal ordering because there's never an opportunity for reordering - each item is emitted immediately before the next arrives.

**Principle**: Temporal ordering is only observable when multiple items are in the system simultaneously. Tests must batch sends to exercise the ordering buffer.

## Consequences

### Positive

1. **Predictable Error Handling**: Errors propagate immediately without being blocked by buffered values
2. **No Deadlocks**: The watermark mechanism prevents indefinite buffering
3. **Clear Test Semantics**: Understanding the three-layer model makes test behavior obvious
4. **Performance**: Error bypass path avoids unnecessary buffering overhead

### Negative

1. **Mental Model Complexity**: Developers must understand three distinct layers of processing
2. **Error Positioning Surprises**: Errors may appear "out of order" relative to values (by design)
3. **Testing Discipline Required**: Developers must remember to use batch send/assert patterns
4. **Documentation Burden**: The behavior requires clear explanation to prevent confusion

### Neutral

1. **Watermark Advancement**: Errors serving as watermarks is powerful but non-obvious
2. **FIFO + Ordering**: Hybrid model (per-stream FIFO, cross-stream temporal) requires careful mental tracking

## Alternatives Considered

### Alternative 1: Buffer Errors Like Values
**Rejected** because:
- Errors could be delayed indefinitely waiting for earlier-timestamped values
- Error propagation is critical for fault detection - delays are unacceptable
- Errors lack intrinsic timestamps (unlike values), making temporal positioning ambiguous

### Alternative 2: Errors Terminate Streams
**Rejected** because:
- Contradicts Fluxion's "errors are data" philosophy
- Prevents recovery and continuation after transient failures
- Users can implement termination with `take_while_with` if desired

### Alternative 3: Strict Timestamp Ordering for Errors
**Rejected** because:
- Errors don't have meaningful timestamps (they represent exceptional conditions, not events)
- Would require artificial timestamp assignment to errors
- Increases complexity without clear benefit

## Implementation Notes

### Key Code Locations

- **`fluxion-ordered-merge/src/lib.rs`**: Core merging algorithm with watermark tracking
- **`fluxion-stream/tests/ordered_merge/ordered_merge_error_tests.rs`**: Error propagation tests demonstrating bypass behavior
- **`fluxion-stream/tests/ordered_merge/ordered_merge_tests.rs`**: Temporal ordering tests with batch send/assert patterns

### Watermark Calculation

```rust
// Pseudocode for watermark advancement
watermark = min(
    pending_values.iter().map(|v| v.timestamp()).min(),
    active_streams.iter().filter_map(|s| s.peek_timestamp()).min()
)

// When error arrives:
// 1. Emit error immediately
// 2. Recalculate watermark (error stream no longer contributes a timestamp)
// 3. Emit all buffered values with timestamp ≤ new watermark
```

### Testing Guidelines

1. **Use `test_channel_with_errors`**: Allows sending both values and errors through the same channel
2. **Batch sends before asserts**: Group all `unbounded_send` calls, then group all assertions
3. **Test error bypass explicitly**: Include tests where errors arrive before/between/after values
4. **Document expected emission order**: Use comments to clarify why errors appear at specific positions

## Related Decisions

- **Error Handling Philosophy** (documented in `docs/ERROR-HANDLING.md`): Errors as first-class stream items
- **Temporal Ordering Trait** (documented in `fluxion-core/src/timestamped.rs`): `HasTimestamp` trait design
- **Testing Utilities** (documented in `fluxion-test-utils/README.md`): `Sequenced<T>` for deterministic timestamps

## References

- [Operator Summary: ordered_merge](../FLUXION_OPERATOR_SUMMARY.md#ordered_merge)
- [Error Handling Guide](../ERROR-HANDLING.md)
- [Conversation: January 2026 - Temporal Ordering Deep Dive](https://github.com/umbgtt10/fluxion/discussions/TBD)

## Revision History

- **2026-02-01**: Initial ADR documenting temporal ordering semantics and error bypass behavior
