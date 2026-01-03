# fluxion-exec

> **Part of [Fluxion](../README.md)** - A reactive stream processing library for Rust

Async stream execution utilities that enable processing streams with async handlers, providing fine-grained control over concurrency, cancellation, and error handling.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## Overview

`fluxion-exec` provides two powerful execution patterns for consuming async streams:

- **`subscribe`** - Sequential processing where every item is processed to completion
- **`subscribe_latest`** - Latest-value processing with automatic cancellation of outdated work

These utilities solve the common problem of how to process stream items with async functions while controlling concurrency, managing cancellation, and handling errors gracefully.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Execution Patterns](#execution-patterns)
  - [subscribe - Sequential Processing](#subscribe---sequential-processing)
  - [subscribe_latest - Latest-Value Processing](#subscribe_latest---latest-value-processing)
- [Detailed Examples](#detailed-examples)
- [Use Cases](#use-cases)
- [Performance Characteristics](#performance-characteristics)
- [Error Handling](#error-handling)
- [Cancellation](#cancellation)
- [Common Patterns](#common-patterns)
- [Anti-Patterns](#anti-patterns)
- [Comparison with Alternatives](#comparison-with-alternatives)
- [Troubleshooting](#troubleshooting)
- [API Reference](#api-reference)
- [License](#license)

## Features

✨ **Two Execution Modes**
- Sequential processing - process every item in order
- Latest-value processing - skip intermediate values, process only latest

🎯 **Flexible Error Handling**
- Custom error callbacks
- Error collection and propagation
- Continue processing on errors

🚀 **Async-First Design**
- Built on tokio runtime
- Spawns background tasks for concurrent execution
- Non-blocking stream consumption

⚡ **Cancellation Support**
- Built-in `CancellationToken` integration
- Automatic cancellation of outdated work (in `subscribe_latest`)
- Graceful shutdown support

🔧 **Extension Trait Pattern**
- Works with any `Stream` implementation
- Compose with other stream operators
- Type-safe and ergonomic API

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fluxion-exec = "0.5"
tokio = { version = "1.48", features = ["rt", "sync", "macros"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Quick Start

The following sections contain a wide range of examples and suggestions. These are indicative and should not be expected to compile as they are.
Check the following files for genuine runnable examples that can be used as they are:
 - [subscribe](./tests/subscribe_tests.rs)
 - [subscribe_latest](./tests/subscribe_latest_tests.rs)

### Sequential Processing

```rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx);

    // Spawn processor
    tokio::spawn(async move {
        stream
            .subscribe(
                |item, _ctx| async move {
                    println!("Processing: {}", item);
                    // Simulate async work
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok::<_, std::io::Error>(())
                },
                |_| {},  // No error callback
                None     // No cancellation token
            )
            .await
    });

    // Send items
    tx.send(1)?;
    tx.send(2)?;
    tx.send(3)?;

    Ok(())
}
```

### Latest-Value Processing

```rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = unbounded_channel();
    let stream = UnboundedReceiverStream::new(rx);

    tokio::spawn(async move {
        stream
            .subscribe_latest(
                |item, token| async move {
                    // Check cancellation periodically
                    for i in 0..10 {
                        if token.is_cancelled() {
                            println!("Cancelled processing {}", item);
                            return Ok(());
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    println!("Completed: {}", item);
                    Ok::<_, std::io::Error>(())
                },
                None,
                None
            )
            .await
    });

    // Rapidly send items - intermediate ones will be skipped
    tx.send(1)?;
    tx.send(2)?;
    tx.send(3)?;
    tx.send(4)?;

    Ok(())
}
```

## Core Concepts

### Subscription

A subscription attaches an async handler to a stream and processes items until the stream ends or a cancellation token is triggered.

### Sequential Execution

With `subscribe`, items are processed one at a time. Each item's handler must complete before the next item is processed. This guarantees:
- Every item is processed
- Processing order is maintained
- No concurrent execution of handlers

### Latest-Value Processing

With `subscribe_latest`, only the most recent value is processed. When new items arrive during processing:
- Current processing continues
- Latest item is queued
- Intermediate items are discarded
- After completion, the latest queued item is processed

This is ideal for scenarios where:
- Only current state matters
- Old values become irrelevant
- Expensive operations should skip stale data

## Execution Patterns

### subscribe - Sequential Processing

**Process every item in order with async handlers.**

```rust

stream.subscribe(
    |item, cancellation_token| async move {
        // Your async processing logic
        process_item(item).await?;
        Ok::<_, MyError>(())
    },
    |error| eprintln!("Error: {:?}", error), // Error callback
    Some(cancellation_token) // Optional cancellation
).await?;
```

**When to use:**
- Every item must be processed (e.g., database writes, event logging)
- Processing order matters
- Side effects must occur for each item
- Work cannot be skipped

**Examples:**
- Writing audit logs
- Processing financial transactions
- Sending notifications
- Persisting events to database

### subscribe_latest - Latest-Value Processing

**Process only the latest value, canceling work for outdated items.**

```rust

stream.subscribe_latest(
    |item, cancellation_token| async move {
        // Check cancellation periodically in long-running tasks
        for chunk in work_chunks {
            if cancellation_token.is_cancelled() {
                return Ok(()); // Gracefully exit
            }
            process_chunk(chunk).await?;
        }
        Ok::<_, MyError>(())
    },
    Some(|error| eprintln!("Error: {:?}", error)),
    Some(cancellation_token)
).await?;
```

**When to use:**
- Only latest value matters (e.g., UI rendering, auto-save)
- Old values become irrelevant
- Expensive operations should skip intermediate values
- Real-time updates

**Examples:**
- Rendering UI with latest state
- Search-as-you-type queries
- Live preview updates
- Auto-saving current document

## Detailed Examples

### Example 1: Database Event Processing

Process every event sequentially and persist to database:

```rust

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: u64,
    data: String,
}

async fn save_to_db(event: &Event) -> Result<(), Box<dyn std::error::Error>> {
    // Database operation
    println!("Saving event {} to database", event.id);
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let handle = tokio::spawn(async move {
        stream
            .subscribe(
                |event, _| async move {
                    save_to_db(&event).await?;
                    Ok::<_, Box<dyn std::error::Error>>(())
                },
                |err| eprintln!("Failed to save event: {}", err),
                None
            )
            .await
    });

    // Generate events
    for i in 0..10 {
        tx.send(Event { id: i, data: format!("Event {}", i) })?;
    }
    drop(tx);

    handle.await??;
    Ok(())
}
```

### Example 2: Search-As-You-Type

Cancel outdated searches when new queries arrive:

```rust

async fn search_api(query: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    println!("Searching for: {}", query);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    Ok(vec![format!("Result for {}", query)])
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let handle = tokio::spawn(async move {
        stream
            .subscribe_latest(
                |query: String, token| async move {
                    if token.is_cancelled() {
                        println!("Query '{}' cancelled", query);
                        return Ok(());
                    }

                    let results = search_api(&query).await?;

                    if !token.is_cancelled() {
                        println!("Results for '{}': {:?}", query, results);
                    }

                    Ok::<_, Box<dyn std::error::Error>>(())
                },
                None,
                None
            )
            .await
    });

    // User types rapidly
    tx.send("r".to_string())?;
    tx.send("ru".to_string())?;
    tx.send("rus".to_string())?;
    tx.send("rust".to_string())?; // Only this search completes
    drop(tx);

    handle.await??;
    Ok(())
}
```

### Example 3: Error Handling and Recovery

```rust

#[derive(Debug)]
struct ProcessingError(String);
impl core::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ProcessingError: {}", self.0)
    }
}
impl std::error::Error for ProcessingError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    let error_count = Arc::new(Mutex::new(0));
    let error_count_clone = error_count.clone();

    let handle = tokio::spawn(async move {
        stream
            .subscribe(
                |item: i32, _| async move {
                    if item % 3 == 0 {
                        return Err(ProcessingError(format!("Cannot process {}", item)));
                    }
                    println!("Processed: {}", item);
                    Ok(())
                },
                move |err| {
                    eprintln!("Error occurred: {}", err);
                    let count = error_count_clone.clone();
                    tokio::spawn(async move {
                        *count.lock().await += 1;
                    });
                },
                None
            )
            .await
    });

    for i in 1..=10 {
        tx.send(i)?;
    }
    drop(tx);

    handle.await??;
    println!("Total errors: {}", *error_count.lock().await);
    Ok(())
}
```

### Example 4: Graceful Shutdown with Cancellation

```rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = futures::channel::mpsc::unbounded();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    let cancel_token = CancellationToken::new();
    let cancel_clone = cancel_token.clone();

    // Spawn shutdown handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("Shutting down gracefully...");
        cancel_clone.cancel();
    });

    let handle = tokio::spawn(async move {
        stream
            .subscribe(
                |item: i32, token| async move {
                    if token.is_cancelled() {
                        println!("Processing cancelled for item {}", item);
                        return Ok(());
                    }

                    println!("Processing: {}", item);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<_, std::io::Error>(())
                },
                |_| {},
                Some(cancel_token)
            )
            .await
    });

    for i in 0..100 {
        if tx.send(i).is_err() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    handle.await??;
    Ok(())
}
```

## Use Cases

### Sequential Processing (`subscribe`)

| Use Case | Description |
|----------|-------------|
| **Event Logging** | Write every event to logs/database |
| **Transaction Processing** | Process financial transactions in order |
| **Message Queue Consumer** | Consume and acknowledge every message |
| **Audit Trail** | Maintain complete audit history |
| **Batch ETL** | Extract, transform, load data sequentially |
| **Notification Service** | Send every notification |
| **File Processing** | Process every file in a directory |

### Latest-Value Processing (`subscribe_latest`)

| Use Case | Description |
|----------|-------------|
| **UI Rendering** | Render only the latest application state |
| **Auto-Save** | Save current document (skip intermediate edits) |
| **Live Preview** | Update preview with latest content |
| **Search Suggestions** | Show results for latest query only |
| **Real-time Dashboard** | Display current metrics |
| **Configuration Reload** | Apply latest config changes |
| **Debounced API Calls** | Call API with latest parameters |

## Performance Characteristics

### Sequential Processing (`subscribe`)

- **Latency**: $O(n \times t)$ where $n$ is number of items, $t$ is processing time
- **Throughput**: Limited by handler execution time
- **Memory**: $O(1)$ - processes one item at a time
- **Ordering**: Strict sequential order maintained
- **Guarantees**: Every item processed exactly once

**Best for**: Correctness and completeness over speed

### Latest-Value Processing (`subscribe_latest`)

- **Latency**: $O(t)$ for latest item (intermediate items skipped)
- **Throughput**: Higher than sequential (skips work)
- **Memory**: $O(1)$ - one active task, one queued item
- **Ordering**: Processes latest available
- **Guarantees**: At most 2 items in flight (current + latest)

**Best for**: Responsiveness and efficiency over completeness

## Error Handling

### With Error Callback

Errors are passed to the callback, processing continues:

```rust
stream.subscribe(
    |item, _| async move {
        risky_operation(item).await?;
        Ok::<_, MyError>(())
    },
    |error| {
        eprintln!("Error: {}", error);
        // Log to monitoring service
        // Increment error counter
        // etc.
    },
    None
).await?; // Returns Ok(()) even if individual items failed
```

### Ignoring Errors

If you want to ignore errors and continue processing:

```rust
use fluxion_exec::ignore_errors;

stream.subscribe(handler, ignore_errors, None).await?;
// All items processed, errors are silently ignored
```

### Fail-Fast Pattern

Return error immediately to stop processing:

```rust
stream.subscribe(
    |item, _| async move {
        critical_operation(item).await?;
        Ok::<_, CriticalError>(())
    },
    None,
    None
).await?; // Stops on first error
```

## Cancellation

### Using CancellationToken

```rust

let cancel_token = CancellationToken::new();
let cancel_clone = cancel_token.clone();

// Start processing
let handle = tokio::spawn(async move {
    stream.subscribe(
        |item, token| async move {
            if token.is_cancelled() {
                return Ok(()); // Exit early
            }
            process(item).await
        },
        Some(cancel_token),
        None
    ).await
});

// Cancel from another task
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(10)).await;
    cancel_clone.cancel();
});
```

### Automatic Cancellation in `subscribe_latest`

The cancellation token passed to handlers is automatically cancelled when newer items arrive:

```rust
stream.subscribe_latest(
    |item, token| async move {
        for i in 0..100 {
            if token.is_cancelled() {
                println!("Item {} cancelled", item);
                return Ok(());
            }
            // Do work...
        }
        Ok(())
    },
    None,
    None
).await?;
```

## Common Patterns

### Pattern: Retry on Failure

```rust

stream.subscribe(
    |item, _| async move {
        let mut attempts = 0;
        loop {
            match process_with_retry(&item).await {
                Ok(()) => return Ok(()),
                Err(e) if attempts < 3 => {
                    attempts += 1;
                    eprintln!("Retry {} for item {:?}: {}", attempts, item, e);
                    tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                }
                Err(e) => return Err(e),
            }
        }
    },
    |_| {},
    None
).await?;
```

### Pattern: Rate Limiting

```rust

let last_process = Arc::new(Mutex::new(Instant::now()));

stream.subscribe(
    move |item, _| {
        let last = last_process.clone();
        async move {
            let mut last_instant = last.lock().await;
            let elapsed = last_instant.elapsed();
            if elapsed < Duration::from_millis(100) {
                tokio::time::sleep(Duration::from_millis(100) - elapsed).await;
            }
            *last_instant = Instant::now();
            drop(last_instant);

            process(item).await
        }
    },
    |_| {},
    None
).await?;
```

### Pattern: Batch Processing

```rust

stream
    .chunks(100)  // Batch 100 items
    .subscribe(
        |batch, _| async move {
            process_batch(&batch).await?;
            Ok::<_, MyError>(())
        },
        |_| {},
        None
    )
    .await?;
```

### Pattern: Conditional Processing

```rust
stream
    .filter(|item| futures::future::ready(item.is_important()))
    .subscribe(
        |item, _| async move {
            process_important(item).await
        },
        |_| {},
        None
    )
    .await?;
```

## Anti-Patterns

### ❌ Don't: Use `subscribe_latest` for Critical Work

```rust
// BAD: Payment processing might be skipped!
payment_stream.subscribe_latest(
    |payment, _| async move {
        process_payment(payment).await  // Could be cancelled!
    },
    None,
    None
).await?;
```

✅ **Good**: Use `subscribe` for work that must complete:

```rust
payment_stream.subscribe(
    |payment, _| async move {
        process_payment(payment).await  // Every payment processed
    },
    |_| {},
    None
).await?;
```

### ❌ Don't: Block in Async Handlers

```rust
// BAD: Blocking operations stall the executor
stream.subscribe(
    |item, _| async move {
        std::thread::sleep(Duration::from_secs(1));  // Blocks executor!
        Ok(())
    },
    None,
    None
).await?;
```

✅ **Good**: Use async operations:

```rust
stream.subscribe(
    |item, _| async move {
        tokio::time::sleep(Duration::from_secs(1)).await;  // Non-blocking
        Ok(())
    },
    |_| {},
    None
).await?;
```

### ❌ Don't: Perform CPU-Intensive Work on Async Runtime

```rust
// BAD: CPU work blocks async tasks
stream.subscribe(
    |data, _| async move {
        expensive_computation(data);  // Blocks!
        Ok(())
    },
    |_| {},
    None
).await?;
```

✅ **Good**: Use `spawn_blocking` for CPU work:

```rust
stream.subscribe(
    |data, _| async move {
        tokio::task::spawn_blocking(move || {
            expensive_computation(data)
        }).await??;
        Ok::<_, Box<dyn std::error::Error>>(())
    },
    |_| {},
    None
).await?;
```

### ❌ Don't: Ignore Cancellation Tokens

```rust
// BAD: Long-running work that can't be cancelled
stream.subscribe_latest(
    |item, _token| async move {  // Token ignored!
        for i in 0..1000000 {
            expensive_step(i).await;
        }
        Ok(())
    },
    None,
    None
).await?;
```

✅ **Good**: Check cancellation periodically:

```rust
stream.subscribe_latest(
    |item, token| async move {
        for i in 0..1000000 {
            if token.is_cancelled() {
                return Ok(());  // Exit early
            }
            expensive_step(i).await;
        }
        Ok(())
    },
    None,
    None
).await?;
```

## Comparison with Alternatives

### vs `futures::StreamExt::for_each`

| Feature | `subscribe` | `for_each` |
|---------|-------------------|------------|
| **Execution** | Spawns tasks | Inline execution |
| **Cancellation** | Built-in token support | Manual |
| **Error handling** | Callbacks + collection | Returns first error |
| **Concurrency** | Configurable | Sequential only |

### vs `futures::StreamExt::buffer_unordered`

| Feature | `subscribe` | `buffer_unordered` |
|---------|-------------------|--------------------|
| **Ordering** | Sequential | Unordered |
| **Concurrency** | One at a time | N concurrent |
| **Backpressure** | Automatic | Manual (buffer size) |
| **Use case** | Sequential processing | Parallel processing |

### vs Manual Task Spawning

| Feature | `subscribe_latest` | Manual spawning |
|---------|-------------------------|------------------|
| **Cancellation** | Automatic | Manual |
| **Latest-value** | Built-in | Manual tracking |
| **Error handling** | Integrated | Manual |
| **Cleanup** | Automatic | Manual |

## Troubleshooting

### Problem: Handler Never Completes

**Symptoms**: Stream processing hangs indefinitely

**Causes**:
- Awaiting on an infinite stream without cancellation
- Deadlock in handler
- Blocking operation in async context

**Solutions**:
```rust
// Add timeout
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    stream.subscribe(handler, None, None)
).await??;

// Use cancellation token
let cancel = CancellationToken::new();
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(30)).await;
    cancel.cancel();
});
```

### Problem: High Memory Usage

**Symptoms**: Memory grows unbounded during processing

**Causes**:
- Stream produces items faster than they can be processed
- Large items held in memory
- Memory leaks in handler

**Solutions**:
```rust
// Add backpressure with bounded channels
let (tx, rx) = futures::channel::mpsc::channel(100); // Bounded

// Use subscribe_latest to skip items
stream.subscribe_latest(handler, None, None).await?;

// Process in batches and clear
stream.chunks(100).subscribe(
    |mut batch, _| async move {
        process(&batch).await?;
        batch.clear(); // Free memory
        Ok(())
    },
    None,
    None
).await?;
```

### Problem: Errors Not Propagated

**Symptoms**: Errors occur but are silently ignored

**Cause**: Error callback provided, but errors not handled

**Solution**:
```rust
let errors = Arc::new(Mutex::new(Vec::new()));
let errors_clone = errors.clone();

stream.subscribe(
    handler,
    None,
    Some(move |err| {
        let errors = errors_clone.clone();
        tokio::spawn(async move {
            errors.lock().await.push(err.to_string());
        });
    })
).await?;

// Check errors after processing
let all_errors = errors.lock().await;
if !all_errors.is_empty() {
    eprintln!("Errors occurred: {:?}", all_errors);
}
```

### Problem: Items Processed Out of Order

**Symptoms**: Items appear in unexpected order

**Cause**: Using concurrent processing patterns

**Solution**: Use `subscribe` (strictly sequential) instead of `buffer_unordered` or parallel patterns

## API Reference

See the [full API documentation](https://docs.rs/fluxion-exec) for detailed type signatures and additional examples.

### Core Traits

- [`SubscribeExt`](https://docs.rs/fluxion-exec/latest/fluxion_exec/trait.SubscribeExt.html) - Sequential processing
- [`SubscribeLatestExt`](https://docs.rs/fluxion-exec/latest/fluxion_exec/trait.SubscribeLatestExt.html) - Latest-value processing

### Related Crates

- [`fluxion-stream`](../fluxion-stream) - Stream composition operators
- [`fluxion-core`](../fluxion-core) - Core types and errors

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE.md) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../LICENSE-MIT.md) or http://opensource.org/licenses/MIT)

at your option.

Copyright 2025 Umberto Gotti
