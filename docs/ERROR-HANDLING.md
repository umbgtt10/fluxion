# Error Handling in Fluxion

This guide explains Fluxion's error handling philosophy and provides patterns for handling errors effectively in stream processing applications.

## Philosophy

Fluxion follows a **propagate, don't hide** approach to error handling:

- **Errors are first-class values** - They flow through streams like any other data
- **No silent failures** - Internal errors (like lock contention) are surfaced to users
- **Composable error handling** - Errors can be handled at any point in the stream pipeline
- **Type-safe recovery** - The type system guides proper error handling

This approach gives you full visibility into what's happening in your stream processing pipeline and allows you to make informed decisions about error recovery.

## Core Concepts

### StreamItem<T>

All stream operators in Fluxion return `StreamItem<T>`, which represents either a successful value or an error:

```rust
pub enum StreamItem<T> {
    Value(T),
    Error(FluxionError),
}
```

This is similar to `Result<T, FluxionError>` but optimized for streaming contexts where errors should flow alongside values rather than short-circuit execution.

### FluxionError

The root error type encompasses all failure modes in Fluxion:

```rust
pub enum FluxionError {
    LockError { context: String },
    StreamProcessingError { context: String },
    UserError(Box<dyn Error + Send + Sync>),
    MultipleErrors { count: usize, errors: Vec<FluxionError> },
}
```

**Variant descriptions:**

- **`LockError`**: Lock acquisition failed (poisoned mutex). Usually transient and recoverable.
- **`StreamProcessingError`**: Internal stream processing error. Usually permanent, indicates serious issue.
- **`UserError`**: User-provided callback or operation failed. Wrap your domain errors here.
- **`MultipleErrors`**: Multiple errors occurred (e.g., in `subscribe`). Check the `errors` vector for details.

See the [FluxionError API documentation](https://docs.rs/fluxion-core) for detailed descriptions of each variant.

## Error Handling Patterns

Fluxion provides several patterns for handling errors in stream processing:

### Using the `on_error` Operator

The `on_error` operator provides composable, selective error handling:

#### Basic Error Consumption

```rust
use fluxion_stream::OnErrorExt;
use fluxion_core::FluxionError;

let stream = source_stream
    .on_error(|err| {
        log::error!("Stream error: {}", err);
        true // Consume all errors
    });
```

#### Chain of Responsibility Pattern

```rust
let stream = source_stream
    .on_error(|err| {
        // Handle validation errors
        if err.to_string().contains("validation") {
            metrics::increment("validation_errors");
            log::warn!("Validation error: {}", err);
            true // Consume
        } else {
            false // Pass to next handler
        }
    })
    .on_error(|err| {
        // Handle network errors
        if err.to_string().contains("network") {
            metrics::increment("network_errors");
            log::error!("Network error: {}", err);
            true // Consume
        } else {
            false // Pass to next handler
        }
    })
    .on_error(|err| {
        // Catch-all for unhandled errors
        log::error!("Unhandled error: {}", err);
        metrics::increment("unhandled_errors");
        true // Consume all remaining errors
    });
```

See [on_error operator specification](FLUXION_OPERATOR_SUMMARY.md#on_error) for complete details.

### Pattern Matching on StreamItem

Match on each stream item to handle values and errors separately:

```rust
use fluxion_stream::{IntoFluxionStream, CombineLatestExt};
use fluxion_core::StreamItem;
use futures::StreamExt;

let stream = rx.into_fluxion_stream();
let mut combined = stream.combine_latest(vec![other_stream], |_| true);

while let Some(item) = combined.next().await {
    match item {
        StreamItem::Value(data) => {
            // Process successful value
            println!("Data: {:?}", data);
        }
        StreamItem::Error(e) => {
            // Handle lock error or other failures
            eprintln!("Stream error: {}", e);
            // Decide: continue processing, retry, or abort?
        }
    }
}
```

## Common Error Scenarios

### Lock Contention

Stream operators use internal locks for shared state. If a lock becomes poisoned (thread panicked while holding it), a `LockError` is emitted. Use pattern matching or `on_error` to handle these:

```rust
use fluxion_stream::{IntoFluxionStream, CombineLatestExt};
use fluxion_core::StreamItem;
use futures::StreamExt;

let stream = rx.into_fluxion_stream();
let mut combined = stream.combine_latest(vec![other_stream], |_| true);

while let Some(item) = combined.next().await {
    match item {
        StreamItem::Value(data) => {
            // Process successful value
            println!("Data: {:?}", data);
        }
        StreamItem::Error(e) => {
            // Handle lock error or other failures
            eprintln!("Stream error: {}", e);
            // Decide: continue processing, retry, or abort?
        }
    }
}
```

### Channel Failures

When channels close unexpectedly or send operations fail, stream processing continues gracefully:

```rust
use fluxion_rx::prelude::*;

let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
let stream = rx.into_fluxion_stream();

// If tx is dropped, stream will end gracefully
// Channel errors are handled internally by the stream
```

### User Callback Errors

Errors from user-provided closures are wrapped and propagated:

```rust
use fluxion_exec::SubscribeExt;

let result = stream
    .subscribe(
        |item| async move {
            // User code that might fail
            process_item(item).await?;
            Ok(())
        },
        None,
        None,
    )
    .await;

// Errors from process_item are aggregated into MultipleErrors
match result {
    Ok(()) => println!("All items processed successfully"),
    Err(FluxionError::MultipleErrors { count, errors }) => {
        eprintln!("{} errors occurred:", count);
        for err in errors.iter().take(10) {
            eprintln!("  - {}", err);
        }
    }
    Err(e) => eprintln!("Subscription failed: {}", e),
}
```

## Error Handling Patterns

### Pattern 1: Unwrap When Errors Are Impossible

If you know errors cannot occur (e.g., simple transformations without shared state), unwrap is appropriate:

```rust
let mapped = stream
    .map_ordered(|x| x * 2)
    .map(|item| item.unwrap()); // Safe: map_ordered never fails

let result = mapped.collect::<Vec<_>>().await;
```

### Pattern 2: Filter and Log Errors

Process values and log errors without stopping the stream:

```rust
use futures::StreamExt;

let stream = stream
    .combine_latest(vec![other], |_| true)
    .filter_map(|item| {
        match item {
            StreamItem::Value(v) => Some(v),
            StreamItem::Error(e) => {
                tracing::warn!("Stream error (continuing): {}", e);
                None
            }
        }
    });
```

### Pattern 3: Collect Errors for Later

Separate values and errors for different handling:

```rust
let mut values = Vec::new();
let mut errors = Vec::new();

while let Some(item) = stream.next().await {
    match item {
        StreamItem::Value(v) => values.push(v),
        StreamItem::Error(e) => errors.push(e),
    }
}

if !errors.is_empty() {
    eprintln!("Encountered {} errors", errors.len());
    // Handle errors...
}

// Process values...
```

### Pattern 4: Short-Circuit on First Error

Stop processing as soon as an error occurs:

```rust
use futures::StreamExt;

let result: Result<Vec<_>, FluxionError> = stream
    .map(|item| match item {
        StreamItem::Value(v) => Ok(v),
        StreamItem::Error(e) => Err(e),
    })
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect();

match result {
    Ok(values) => println!("All items: {:?}", values),
    Err(first_error) => eprintln!("Failed: {}", first_error),
}
```

### Pattern 5: Retry with Backoff

Implement retry logic for transient failures:

```rust
use tokio::time::{sleep, Duration};

let mut stream = stream.combine_latest(vec![other], |_| true);

while let Some(item) = stream.next().await {
    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        match &item {
            StreamItem::Value(data) => {
                process(data);
                break;
            }
            StreamItem::Error(e) if attempts < max_attempts => {
                attempts += 1;
                tracing::warn!("Error (attempt {}/{}): {}", attempts, max_attempts, e);
                sleep(Duration::from_millis(100 * attempts)).await;
                // Could recreate stream or retry operation
            }
            StreamItem::Error(e) => {
                tracing::error!("Failed after {} attempts: {}", max_attempts, e);
                break;
            }
        }
    }
}
```

## Operator-Specific Error Conditions

### combine_latest

**Can fail when:**
- Lock acquisition fails (poisoned mutex)
- Internal state becomes inconsistent

**Recommendation:** Handle `LockError` by logging and continuing or restarting the stream

### with_latest_from

**Can fail when:**
- Lock acquisition fails
- Secondary stream state cannot be accessed

**Recommendation:** Similar to `combine_latest` - transient errors can be logged and ignored

### ordered_merge

**Can fail when:**
- Internal buffering encounters issues
- Lock contention occurs

**Recommendation:** Generally reliable; errors indicate serious issues that may require stream restart

### take_latest_when / emit_when / take_while_with

**Can fail when:**
- Lock acquisition fails
- Filter stream state cannot be accessed

**Recommendation:** These typically have transient errors; consider retry logic

### subscribe / subscribe_latest

**Can fail when:**
- User callback returns an error
- Multiple items fail and no error callback is provided

**Recommendation:** Always check the `Result` return value and handle `MultipleErrors`

## Testing Error Paths

When writing tests, explicitly test error conditions:

```rust
#[tokio::test]
async fn test_handles_lock_error() {
    // Setup: Create scenario that might cause lock error
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let stream = rx.into_fluxion_stream();

    // Act: Trigger potential error
    let mut combined = stream.combine_latest(vec![other_stream], |_| true);

    // Assert: Verify error handling
    let mut error_count = 0;
    while let Some(item) = combined.next().await {
        if matches!(item, StreamItem::Error(_)) {
            error_count += 1;
        }
    }

    // Verify errors were handled appropriately
}
```

## Error Recovery Strategies

### 1. Continue Processing (Log and Skip)

Best for: Non-critical errors, monitoring scenarios

```rust
stream.filter_map(|item| match item {
    StreamItem::Value(v) => Some(v),
    StreamItem::Error(e) => {
        metrics::increment_counter!("stream_errors");
        None
    }
})
```

### 2. Graceful Degradation

Best for: Services that should stay up despite errors

```rust
stream.map(|item| match item {
    StreamItem::Value(v) => v,
    StreamItem::Error(_) => Default::default(), // Use fallback value
})
```

### 3. Circuit Breaker

Best for: Protecting downstream services

```rust
let mut error_count = 0;
const ERROR_THRESHOLD: usize = 10;

while let Some(item) = stream.next().await {
    match item {
        StreamItem::Value(v) => {
            error_count = 0;
            process(v);
        }
        StreamItem::Error(e) => {
            error_count += 1;
            if error_count >= ERROR_THRESHOLD {
                eprintln!("Circuit breaker opened after {} errors", error_count);
                break; // Stop processing
            }
        }
    }
}
```

### 4. Dead Letter Queue

Best for: Critical data that must not be lost

```rust
let (dlq_tx, dlq_rx) = tokio::sync::mpsc::unbounded_channel();

stream.for_each(|item| async {
    match item {
        StreamItem::Value(v) => process(v),
        StreamItem::Error(e) => {
            // Send to dead letter queue for later processing
            dlq_tx.send((item_data, e)).ok();
        }
    }
}).await;
```

## Best Practices

1. **Always handle errors at the boundary** - Don't let `StreamItem::Error` leak into non-Fluxion code
2. **Use typed errors** - Wrap your domain errors in `FluxionError::UserError`
3. **Log errors with context** - Include enough information to debug issues
4. **Test error paths** - Write tests that explicitly verify error handling
5. **Document error conditions** - Use `# Errors` sections in your API docs
6. **Consider error budgets** - Decide how many errors are acceptable before taking action
7. **Monitor error rates** - Track errors in production to detect issues early

## Migration from Panic-Based Code

If you're migrating code that previously panicked on errors:

**Before (panics on error):**
```rust
let guard = state.lock().unwrap(); // Panics if poisoned
```

**After (propagates error):**
```rust
match lock_or_error(&state, "operation_name") {
    Ok(guard) => { /* process */ },
    Err(e) => return Some(StreamItem::Error(e)),
}
```

This change allows your application to:
- Continue processing other items
- Implement retry logic
- Collect statistics on error rates
- Gracefully degrade instead of crashing

## See Also

- [FluxionError API Documentation](https://docs.rs/fluxion-core)
- [ROADMAP.md](../ROADMAP.md) - Roadmap to 1.0 release
- [Stream Operators](FLUXION_OPERATOR_SUMMARY.md) - Complete operator reference
- [Examples](../examples/) - Working code showing error handling patterns
