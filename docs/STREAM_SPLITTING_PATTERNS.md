# Stream Splitting Patterns in FluxionStream

## Overview

This document explores patterns for splitting a single FluxionStream into multiple independent streams that can be consumed separately. Due to Rust's ownership model, FluxionStream operators consume `self`, making direct cloning impossible. Stream splitting always requires spawning a task to forward items to multiple consumers.

## Fundamental Constraint

**Task spawning is unavoidable for stream splitting in Rust.**

When you want multiple consumers of a single stream:
- Rust's ownership prevents holding multiple references
- A forwarding task is required to multiplex the stream data
- This is not a limitation but a fundamental architectural requirement

## Two Main Approaches

### 1. Extension with Internal Task (Post-Chain Splitting)

Split an already-chained stream into multiple branches:

```rust
// Chain operators first
let stream = FluxionStream::from_unbounded_receiver(rx)
    .map(|x| x * 2)
    .filter(|x| x > 10);

// Then split into independent branches
let (stream_a, stream_b) = stream.split();

// Each branch can have different processing
let branch_a = stream_a
    .map(|x| format!("A: {}", x))
    .subscribe(|msg| println!("{}", msg));

let branch_b = stream_b
    .filter(|x| x % 2 == 0)
    .subscribe(|count| total += count);
```

#### Implementation Pattern

```rust
pub trait SplitExt<T>: Stream<Item = StreamItem<T>> + Sized + Send + 'static
where
    T: Clone + Send + 'static,
{
    fn split(self) -> (FluxionStream<impl Stream<Item = StreamItem<T>>>,
                       FluxionStream<impl Stream<Item = StreamItem<T>>>) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            futures::pin_mut!(self);
            while let Some(item) = self.next().await {
                let item_clone = item.clone();

                // If either send fails, the receiver was dropped
                if tx1.send(item).is_err() || tx2.send(item_clone).is_err() {
                    break;
                }
            }
        });

        (
            FluxionStream::from_unbounded_receiver(rx1),
            FluxionStream::from_unbounded_receiver(rx2),
        )
    }

    fn split_with_handle(self) -> (
        FluxionStream<impl Stream<Item = StreamItem<T>>>,
        FluxionStream<impl Stream<Item = StreamItem<T>>>,
        JoinHandle<()>,
    ) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            futures::pin_mut!(self);
            while let Some(item) = self.next().await {
                let item_clone = item.clone();

                if tx1.send(item).is_err() || tx2.send(item_clone).is_err() {
                    break;
                }
            }
        });

        (
            FluxionStream::from_unbounded_receiver(rx1),
            FluxionStream::from_unbounded_receiver(rx2),
            handle,
        )
    }
}
```

#### Pros
- **Ergonomic**: Works with any existing stream mid-chain
- **Flexible**: Chain first, decide to split later
- **Simple API**: Just call `.split()` when needed
- **Automatic cleanup**: Task exits when consumers drop

#### Cons
- **Runtime overhead**: New task spawned at split point
- **Clone requirement**: `T: Clone` needed to duplicate items
- **Per-split cost**: Each split creates its own forwarding task

#### When to Use
- Unknown consumer count until runtime
- Need to split after applying common transformations
- Splitting is rare/occasional in your application
- Prioritize API ergonomics over maximum performance

---

### 2. At-Source Broadcasting (Pre-Chain Splitting)

Create multiple streams from a single source before chaining:

```rust
let (tx, _) = tokio::sync::broadcast::channel(100);
let rx1 = tx.subscribe();
let rx2 = tx.subscribe();

// Single task forwards from source to broadcast
tokio::spawn(async move {
    while let Some(item) = source_stream.next().await {
        let _ = tx.send(item); // All subscribers receive
    }
});

// Each subscriber gets independent FluxionStream
let stream_a = FluxionStream::from_broadcast_receiver(rx1)
    .map(|x| format!("A: {}", x));

let stream_b = FluxionStream::from_broadcast_receiver(rx2)
    .filter(|x| x % 2 == 0);
```

#### Implementation Pattern

Already exists in `wrapped_stream_broadcast.rs`:

```rust
impl<T> FluxionStream<impl Stream<Item = StreamItem<T>>>
where
    T: Clone + Send + 'static,
{
    pub fn from_broadcast_receiver(
        mut rx: tokio::sync::broadcast::Receiver<T>,
    ) -> Self {
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(value) => yield StreamItem::Item(value),
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        yield StreamItem::Error(format!("Lagged by {} messages", n));
                    }
                }
            }
        };
        FluxionStream { stream }
    }
}
```

#### Pros
- **Efficient**: Single task for N consumers
- **Scalable**: Adding consumers doesn't add tasks
- **Built-in backpressure**: Broadcast channels handle slow consumers
- **Lower overhead**: One task instead of multiple

#### Cons
- **Requires planning**: Must know you need multiple consumers upfront
- **Can't split mid-chain**: Broadcasting happens at source
- **Channel configuration**: Need to set buffer size appropriately
- **Lagging messages**: Slow consumers can miss items

#### When to Use
- Known multiple consumers from the start
- High-throughput scenarios with many consumers
- Source stream is expensive to compute
- Want to share raw data, apply different transforms per consumer

---

## Comparison Matrix

| Aspect | Extension (Post-Chain) | At-Source (Pre-Chain) |
|--------|------------------------|------------------------|
| **Timing** | Split after chaining | Split before chaining |
| **Task count** | One per split | One per source |
| **Flexibility** | High - split anywhere | Low - must plan ahead |
| **Overhead** | Higher for many splits | Lower for many consumers |
| **API** | `.split()` method | `broadcast::channel()` |
| **Cleanup** | Automatic | Automatic |
| **Common transforms** | Before split | After split (per consumer) |
| **Use case** | Conditional splitting | Known multi-consumer |

---

## Task Lifecycle Management

### Automatic Cleanup (Recommended)

```rust
let (stream_a, stream_b) = stream.split();

// Task automatically exits when:
// 1. Source stream ends (normal completion)
// 2. All receivers are dropped (consumers done)
// 3. Any send fails (receiver dropped)
```

**No explicit management needed** - task cleans up automatically.

### With Monitoring (Advanced)

```rust
let (stream_a, stream_b, handle) = stream.split_with_handle();

// Can monitor or abort if needed
tokio::spawn(async move {
    if let Err(e) = handle.await {
        eprintln!("Split task panicked: {}", e);
    }
});
```

Use when you need:
- Panic detection and recovery
- Explicit task lifecycle control
- Metrics/monitoring on task health

---

## Error Handling

### Error Propagation

```rust
// Errors propagate to ALL branches via StreamItem::Error
let (stream_a, stream_b) = stream.split();

// Both streams receive the error
stream_a.subscribe(|item| match item {
    StreamItem::Item(value) => process(value),
    StreamItem::Error(e) => log_error(&e),
});

stream_b.subscribe(|item| match item {
    StreamItem::Item(value) => process(value),
    StreamItem::Error(e) => log_error(&e),
});
```

### Task Failure Modes

The forwarding task can only "fail" by panicking:

1. **Normal completion**: Source stream ends → task exits cleanly
2. **Consumer dropout**: Receiver dropped → send fails → task exits cleanly
3. **Panic**: Bug in forwarding logic → task crashes

For panics, use `.split_with_handle()` to monitor:

```rust
let (stream_a, stream_b, handle) = stream.split_with_handle();

tokio::spawn(async move {
    match handle.await {
        Ok(()) => println!("Split task completed normally"),
        Err(e) if e.is_panic() => {
            eprintln!("Split task panicked!");
            // Implement recovery logic
        }
        _ => {}
    }
});
```

---

## Performance Considerations

### Memory

- Each split creates new channels (unbounded = unbounded memory growth)
- Items are cloned (N-1 clones for N consumers)
- Broadcast channels have fixed buffer (bounded memory)

### CPU

- Extension: O(N) tasks for N splits
- At-source: O(1) task for N consumers
- Clone cost: O(N × item_size) per item

### Backpressure

- **Extension splits**: No backpressure between branches (unbounded channels)
- **Broadcast**: Built-in backpressure, slow consumers lag or drop messages

---

## Design Recommendations

### Use Extension Split When:
- You don't know if you'll need to split until runtime
- Splitting is conditional or rare
- You want to apply common transforms before splitting
- API ergonomics matter more than maximum performance
- Consumer count is low (2-3 branches)

### Use At-Source Broadcast When:
- You know you need multiple consumers from the start
- High throughput with many consumers (5+ branches)
- Source stream is expensive and shouldn't be duplicated
- You can tolerate message lagging for slow consumers
- Performance is critical

### Hybrid Approach:
```rust
// Start with broadcast for multiple consumers
let (tx, _) = broadcast::channel(100);
let rx1 = tx.subscribe();
let rx2 = tx.subscribe();

spawn(/* forward source to tx */);

// Each consumer can ALSO split if needed
let stream_a = FluxionStream::from_broadcast_receiver(rx1)
    .map(|x| x * 2);

let (stream_a1, stream_a2) = stream_a.split(); // Extension split on broadcast consumer
```

Combine both patterns when you have:
- Multiple primary consumers (broadcast)
- Each primary consumer needs conditional sub-splits (extension)

---

## Conclusion

**Both approaches require task spawning** - this is fundamental to Rust's ownership model, not a design flaw.

Choose based on:
- **When you know you need splitting**: Pre-chain (broadcast) vs post-chain (extension)
- **Consumer count**: Few (extension) vs many (broadcast)
- **Performance profile**: Flexibility (extension) vs efficiency (broadcast)

The extension approach (`.split()`) is recommended for FluxionStream because it preserves the ergonomic chaining API and works with the existing operator ecosystem.
