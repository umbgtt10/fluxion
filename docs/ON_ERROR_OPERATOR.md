# `on_error` Operator Specification

## Overview

The `on_error` operator implements the **Chain of Responsibility pattern** for error handling in Fluxion streams. It allows composable, selective error handling where each handler decides whether to consume or propagate errors.

## Signature

```rust
fn on_error<F>(self, handler: F) -> OnError<Self, F>
where
    F: FnMut(&FluxionError) -> bool;
```

## Behavior

### When handler returns `true` (error matched and handled):
1. **Consume the error** - Remove the `StreamItem::Error` from the stream
2. **Continue processing** - Allow subsequent `StreamItem::Value` items to flow normally
3. **Enable side effects** - Handler can log, send metrics, or perform other actions

### When handler returns `false` (error not matched):
1. **Propagate the error** - Pass `StreamItem::Error` downstream unchanged
2. **Chain to next handler** - Allow subsequent `on_error` operators to handle it

## Usage Patterns

### Basic Error Handling

```rust
use fluxion_rx::FluxionStream;
use fluxion_core::error::FluxionError;

stream
    .on_error(|err| {
        log::error!("Stream error occurred: {}", err);
        true // Consume all errors, continue stream
    })
```

### Selective Error Handling (Chain of Responsibility)

```rust
stream
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

### Error Type Discrimination

```rust
stream
    .on_error(|err| {
        match err {
            FluxionError::User(user_err) => {
                // Handle user errors
                notify_user(&user_err);
                true
            }
            FluxionError::Stream(msg) if msg.contains("timeout") => {
                // Handle timeout errors
                retry_later();
                true
            }
            FluxionError::MultipleErrors(_) => {
                // Handle aggregated errors
                alert_ops_team(err);
                true
            }
            _ => false // Propagate other errors
        }
    })
```

### Conditional Error Suppression

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

## Implementation Details

### Stream Adapter Structure

```rust
pub struct OnError<S, F> {
    stream: S,
    handler: F,
}

impl<S, F> Stream for OnError<S, F>
where
    S: Stream<Item = StreamItem<T>>,
    F: FnMut(&FluxionError) -> bool,
{
    type Item = StreamItem<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.stream) };

        match ready!(stream.poll_next(cx)) {
            Some(StreamItem::Error(err)) => {
                let handler = unsafe { self.get_unchecked_mut() }.handler;

                if handler(&err) {
                    // Error handled, skip it and poll next item
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    // Error not handled, propagate
                    Poll::Ready(Some(StreamItem::Error(err)))
                }
            }
            other => Poll::Ready(other),
        }
    }
}
```

### Trait Extension

```rust
pub trait OnErrorExt: Stream {
    fn on_error<F>(self, handler: F) -> OnError<Self, F>
    where
        Self: Sized,
        F: FnMut(&FluxionError) -> bool,
    {
        OnError {
            stream: self,
            handler,
        }
    }
}

impl<S: Stream> OnErrorExt for S {}
```

## Design Rationale

### Chain of Responsibility Pattern
- **Specific before general** - Like try/catch blocks, handle specific errors first
- **Composable** - Each handler is independent and focused on one error type
- **Explicit propagation** - Clear when errors are consumed vs propagated

### Boolean Return Value
- **Simple** - No complex return types or enums
- **Clear semantics** - `true` = handled, `false` = propagate
- **Flexible** - Handler can perform side effects before returning

### Error Reference
- **Non-consuming** - Handler receives `&FluxionError`, doesn't own it
- **Efficient** - No cloning required
- **Safe** - Error is only dropped if handler returns `true`

## Comparison with Other Approaches

### vs. `catch` (Returns Replacement Value)
```rust
// Other libraries:
stream.catch(|err| default_value()) // Must provide replacement

// Fluxion approach:
stream.on_error(|err| true) // Just consume, no replacement needed
```

### vs. `on_error_resume_next` (Returns Stream)
```rust
// More complex:
stream.on_error_resume_next(|err| fallback_stream())

// Simpler for most cases:
stream.on_error(|err| { log(err); true })
```

### vs. `retry` (Automatic Retry)
```rust
// Automatic retry (future operator):
stream.retry(3)

// Manual error handling:
stream.on_error(|err| {
    if should_retry(err) {
        schedule_retry();
        true
    } else {
        false
    }
})
```

## Error Handling Guarantees

### Stream Continuation
- ✅ Stream continues after error consumption
- ✅ Subsequent values flow normally
- ✅ Multiple errors can be handled independently

### Error Propagation
- ✅ Unhandled errors propagate downstream
- ✅ Error order preserved
- ✅ No errors silently dropped (unless explicitly consumed)

### Side Effects
- ✅ Handler can perform logging, metrics, notifications
- ✅ Side effects execute before error consumption decision
- ✅ No side effects if handler returns `false`

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_on_error_consumes_matched_errors() {
    let stream = create_stream_with_errors();
    let mut handled = stream.on_error(|err| {
        matches!(err, FluxionError::Stream(_))
    });

    // Only non-Stream errors should propagate
    assert!(matches!(next_item(&mut handled).await, StreamItem::Value(_)));
}

#[tokio::test]
async fn test_on_error_propagates_unmatched_errors() {
    let stream = create_stream_with_errors();
    let mut handled = stream.on_error(|_| false);

    // All errors should propagate
    assert!(matches!(next_item(&mut handled).await, StreamItem::Error(_)));
}

#[tokio::test]
async fn test_on_error_chain() {
    let stream = create_stream_with_errors();
    let mut handled = stream
        .on_error(|err| matches!(err, FluxionError::User(_)))
        .on_error(|err| matches!(err, FluxionError::Stream(_)));

    // All errors consumed by chain
    assert!(matches!(next_item(&mut handled).await, StreamItem::Value(_)));
}
```

### Integration Tests
- Error handling in composed streams
- Interaction with other operators
- Performance under error conditions
- Memory safety with panicking handlers

## Performance Considerations

### Overhead
- **Minimal** - Single function call per error
- **No allocations** - Handler receives reference
- **Zero cost when no errors** - Only affects error path

### Optimization Opportunities
- Handler inlining for simple predicates
- Error type discrimination at compile time
- Fast path for catch-all handlers (`|_| true`)

## Future Extensions

### `on_error_resume_next`
```rust
stream.on_error_resume_next(|err| fallback_stream())
```

### `retry`
```rust
stream.retry(3) // Built on top of on_error
```

### `timeout`
```rust
stream.timeout(Duration::from_secs(5))
```

## Migration Path

### From Current Error Handling
```rust
// Before (manual error filtering):
stream
    .filter_ordered(|item| item.is_ok())
    .map_ordered(|item| item.unwrap())

// After (declarative error handling):
stream
    .on_error(|err| { log::error!("{}", err); true })
```

### Integration with Existing Code
- Works with `StreamItem<T>` enum
- Compatible with all existing operators
- No breaking changes required

## Release Target

**Version:** 0.3.0

**Priority:** High (Foundation for error handling operators)

**Dependencies:**
- Existing `FluxionError` type
- Existing `StreamItem<T>` enum
- Stream trait implementation patterns

**Blockers:** None
