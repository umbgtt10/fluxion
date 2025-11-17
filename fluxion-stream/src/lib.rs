// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Stream operators with temporal ordering guarantees.
//!
//! This crate provides reactive stream combinators that maintain temporal ordering
//! across asynchronous operations. All operators work with types implementing the
//! [`Ordered`] trait, which provides an ordering value for correct temporal sequencing.
//!
//! # Architecture
//!
//! The crate is built around several key concepts:
//!
//! - **[`FluxionStream`]**: A wrapper around any `Stream` that provides access to all operators
//! - **[`Ordered`] trait**: Types must have an intrinsic ordering (timestamp, sequence number, etc.)
//! - **Extension traits**: Each operator is provided via an extension trait for composability
//! - **Temporal correctness**: All operators respect the ordering of items across streams
//!
//! ## Operator Categories
//!
//! ### Combination Operators
//!
//! - **[`combine_latest`](CombineLatestExt::combine_latest)**: Emits when any stream emits, combining latest values from all streams
//! - **[`with_latest_from`](WithLatestFromExt::with_latest_from)**: Samples secondary streams only when primary emits
//! - **[`ordered_merge`](OrderedStreamExt::ordered_merge)**: Merges multiple streams preserving temporal order
//!
//! ### Filtering Operators
//!
//! - **[`emit_when`](EmitWhenExt::emit_when)**: Gates source emissions based on filter stream conditions
//! - **[`take_latest_when`](TakeLatestWhenExt::take_latest_when)**: Samples source when filter condition is met
//! - **[`take_while_with`](TakeWhileExt::take_while_with)**: Emits while condition holds, terminates when false
//!
//! ### Transformation Operators
//!
//! - **[`combine_with_previous`](CombineWithPreviousExt::combine_with_previous)**: Pairs each value with previous value
//!
//! # Temporal Ordering Explained
//!
//! All operators in this crate maintain **temporal ordering** - items are processed in the
//! order of their intrinsic ordering value, not the order they arrive at the operator.
//!
//! ## How It Works
//!
//! When multiple streams are combined:
//!
//! 1. Each stream item must implement [`Ordered`], providing a comparable ordering value
//! 2. Operators use [`ordered_merge`](OrderedStreamExt::ordered_merge) internally to sequence items
//! 3. Items are buffered and emitted in order of their ordering value
//! 4. Late-arriving items with earlier ordering are placed correctly in the sequence
//!
//! ## Example: Out-of-Order Delivery
//!
//! ```
//! use fluxion_stream::{FluxionStream, OrderedStreamExt};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
//! let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
//!
//! let stream1 = FluxionStream::from_unbounded_receiver(rx1);
//! let stream2 = FluxionStream::from_unbounded_receiver(rx2);
//!
//! let merged = stream1.ordered_merge(vec![stream2]);
//! let mut merged = merged;
//!
//! // Send out of order - stream2 sends seq=1, stream1 sends seq=2
//! tx2.send(Sequenced::with_sequence(100, 1)).unwrap();
//! tx1.send(Sequenced::with_sequence(200, 2)).unwrap();
//!
//! // Items are emitted in temporal order (seq 1, then seq 2)
//! let first = merged.next().await.unwrap().unwrap();
//! assert_eq!(first.value, 100); // seq=1 arrives first despite being sent second
//! # }
//! ```
//!
//! # Operator Selection Guide
//!
//! Choose the right operator for your use case:
//!
//! ## When You Need Combined State
//!
//! | Operator | Use When | Triggers On | Example Use Case |
//! |----------|----------|-------------|------------------|
//! | [`combine_latest`] | You need latest from all streams | Any stream emits | Dashboard combining multiple data sources |
//! | [`with_latest_from`] | You have primary + context streams | Primary emits only | User clicks enriched with latest config |
//!
//! ## When You Need All Items
//!
//! | Operator | Use When | Output | Example Use Case |
//! |----------|----------|--------|------------------|
//! | [`ordered_merge`] | Merge multiple sources in order | Every item from all streams | Event log from multiple services |
//! | [`combine_with_previous`] | Compare consecutive items | Pairs of (previous, current) | Detecting value changes |
//!
//! ## When You Need Conditional Emission
//!
//! | Operator | Use When | Behavior | Example Use Case |
//! |----------|----------|----------|------------------|
//! | [`emit_when`] | Gate by condition | Emits source when filter is true | Send notifications only when enabled |
//! | [`take_latest_when`] | Sample on condition | Emits latest source when filter triggers | Sample sensor on button press |
//! | [`take_while_with`] | Stop on condition | Emits until condition false, then stops | Process until timeout |
//!
//! # Performance Characteristics
//!
//! ## Memory Usage
//!
//! - **[`combine_latest`]**: $O(n)$ - stores one latest value per stream
//! - **[`ordered_merge`]**: $O(k)$ - buffers items until ordering confirmed ($k$ = buffer size)
//! - **[`with_latest_from`]**: $O(n)$ - stores one value per secondary stream
//! - **[`combine_with_previous`]**: $O(1)$ - stores only previous value
//!
//! ## Latency Considerations
//!
//! - **Ordered operators**: May buffer items waiting for earlier-ordered items
//! - **Unordered operators**: Process items immediately as they arrive
//! - **Combining operators**: Wait for all streams to emit at least once before first emission
//!
//! ## Throughput
//!
//! All operators use lock-free or minimally-locked designs:
//!
//! - Single mutex per operator (not per item)
//! - No blocking operations in hot paths
//! - Efficient polling with `futures::StreamExt`
//!
//! # Return Type Patterns
//!
//! Fluxion operators use three different return type patterns, each chosen for specific
//! reasons related to type erasure, composability, and performance.
//!
//! ## Pattern 1: `impl Stream<Item = T>`
//!
//! **When used:** Lightweight operators with simple transformations
//!
//! **Examples:**
//! - [`ordered_merge`](OrderedStreamExt::ordered_merge)
//! - [`map_ordered`](FluxionStream::map_ordered)
//! - [`filter_ordered`](FluxionStream::filter_ordered)
//!
//! **Benefits:**
//! - Zero-cost abstraction (no boxing)
//! - Compiler can fully optimize the stream pipeline
//! - Type information preserved for further optimizations
//!
//! **Tradeoffs:**
//! - Concrete type exposed in signatures (can be complex)
//! - May increase compile times for deeply nested operators
//!
//! ## Pattern 2: `FluxionStream<impl Stream<Item = T>>`
//!
//! **When used:** Operators that should compose with other FluxionStream methods
//!
//! **Examples:**
//! - [`combine_with_previous`](CombineWithPreviousExt::combine_with_previous)
//! - [`with_latest_from`](WithLatestFromExt::with_latest_from)
//! - [`combine_latest`](CombineLatestExt::combine_latest)
//!
//! **Benefits:**
//! - Enables method chaining with `FluxionStream` convenience methods
//! - Still zero-cost (no boxing)
//! - Provides consistent API surface
//!
//! **Use cases:**
//! - When users are likely to chain multiple operators
//! - When the operator produces a complex transformed type
//!
//! ## Pattern 3: `Pin<Box<dyn Stream<Item = T>>>`
//!
//! **When used:** Operators with dynamic dispatch requirements or complex internal state
//!
//! **Examples:**
//! - [`emit_when`](EmitWhenExt::emit_when)
//! - [`take_latest_when`](TakeLatestWhenExt::take_latest_when)
//! - [`take_while_with`](TakeWhileExt::take_while_with)
//!
//! **Benefits:**
//! - Type erasure simplifies signatures
//! - Reduces compile time for complex operator chains
//! - Hides internal implementation details
//!
//! **Tradeoffs:**
//! - Heap allocation (small overhead)
//! - Dynamic dispatch prevents some optimizations
//! - Runtime cost typically negligible compared to async operations
//!
//! **Why used for these operators:**
//! These operators maintain internal state machines with multiple branches and complex
//! lifetime requirements. Type erasure keeps the public API simple while allowing
//! internal flexibility.
//!
//! ## Choosing the Right Pattern
//!
//! As a user, you typically don't need to worry about these patterns - all three compose
//! seamlessly. For example, combining different operators in a single chain works naturally
//! regardless of their internal implementation patterns. Each operator returns either
//! `impl Stream` or `FluxionStream<impl Stream>`, and they compose transparently.
//!
//! The patterns are implementation details chosen to balance performance, ergonomics,
//! and maintainability.
//!
//! # Common Patterns
//!
//! ## Pattern: Enriching Events with Context
//!
//! Uses [`with_latest_from`](WithLatestFromExt::with_latest_from) to combine a primary stream
//! with context from a secondary stream.
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, WithLatestFromExt};
//! use fluxion_test_utils::Sequenced;
//! use fluxion_core::Ordered;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! // User clicks enriched with latest configuration
//! let (click_tx, click_rx) = tokio::sync::mpsc::unbounded_channel();
//! let (config_tx, config_rx) = tokio::sync::mpsc::unbounded_channel();
//!
//! let clicks = FluxionStream::from_unbounded_receiver(click_rx);
//! let configs = FluxionStream::from_unbounded_receiver(config_rx);
//!
//! let enriched = clicks.with_latest_from(
//!     configs,
//!     |state| state.get_state().clone()
//! );
//! let mut enriched = enriched;
//!
//! // Send config first, then click
//! config_tx.send(Sequenced::with_sequence("theme=dark".to_string(), 1)).unwrap();
//! click_tx.send(Sequenced::with_sequence("button1".to_string(), 2)).unwrap();
//!
//! let result = enriched.next().await.unwrap().unwrap();
//! assert_eq!(result.get().len(), 2); // Has both click and config
//! # }
//! ```
//!
//! ## Pattern: Merging Multiple Event Sources
//!
//! Uses [`ordered_merge`](OrderedStreamExt::ordered_merge) to combine logs from multiple
//! services in temporal order.
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, OrderedStreamExt};
//! use fluxion_test_utils::Sequenced;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! // Combine logs from multiple services in temporal order
//! let (service1_tx, service1_rx) = tokio::sync::mpsc::unbounded_channel();
//! let (service2_tx, service2_rx) = tokio::sync::mpsc::unbounded_channel();
//!
//! let service1 = FluxionStream::from_unbounded_receiver(service1_rx);
//! let service2 = FluxionStream::from_unbounded_receiver(service2_rx);
//!
//! let unified_log = service1.ordered_merge(vec![service2]);
//! let mut unified_log = unified_log;
//!
//! // Send logs with different timestamps
//! service1_tx.send(Sequenced::with_sequence("service1: started".to_string(), 1)).unwrap();
//! service2_tx.send(Sequenced::with_sequence("service2: ready".to_string(), 2)).unwrap();
//!
//! let first = unified_log.next().await.unwrap().unwrap();
//! assert_eq!(first.value, "service1: started");
//! # }
//! ```
//!
//! ## Pattern: Change Detection
//!
//! Uses [`combine_with_previous`](CombineWithPreviousExt::combine_with_previous) to detect
//! when values change by comparing with the previous value.
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, CombineWithPreviousExt};
//! use fluxion_test_utils::Sequenced;
//! use fluxion_core::Ordered;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//! let stream = FluxionStream::from_unbounded_receiver(rx);
//!
//! // Pair each value with its previous value
//! let paired = stream.combine_with_previous();
//! let mut paired = paired;
//!
//! // Send values
//! tx.send(Sequenced::with_sequence(1, 1)).unwrap();
//! tx.send(Sequenced::with_sequence(1, 2)).unwrap(); // Same value
//! tx.send(Sequenced::with_sequence(2, 3)).unwrap(); // Changed!
//!
//! let result = paired.next().await.unwrap().unwrap();
//! assert!(result.previous.is_none()); // First has no previous
//!
//! let result = paired.next().await.unwrap().unwrap();
//! let changed = result.previous.as_ref().unwrap().value != result.current.value;
//! assert!(!changed); // 1 == 1, no change
//!
//! let result = paired.next().await.unwrap().unwrap();
//! let changed = result.previous.as_ref().unwrap().value != result.current.value;
//! assert!(changed); // 1 != 2, changed!
//! # }
//! ```
//!
//! ## Pattern: Conditional Processing
//!
//! Uses [`emit_when`](EmitWhenExt::emit_when) to gate emissions based on a filter stream,
//! only emitting when the condition is satisfied.
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, EmitWhenExt};
//! use fluxion_test_utils::Sequenced;
//! use fluxion_core::Ordered;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! // Send notifications only when enabled
//! let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
//! let (enabled_tx, enabled_rx) = tokio::sync::mpsc::unbounded_channel();
//!
//! let events = FluxionStream::from_unbounded_receiver(event_rx);
//! let enabled = FluxionStream::from_unbounded_receiver(enabled_rx);
//!
//! let notifications = events.emit_when(
//!     enabled,
//!     |state| state.get_state().get(1).map(|v| *v > 0).unwrap_or(false)
//! );
//! let mut notifications = notifications;
//!
//! // Enable notifications
//! enabled_tx.send(Sequenced::with_sequence(1, 1)).unwrap();
//! // Send event
//! event_tx.send(Sequenced::with_sequence(999, 2)).unwrap();
//!
//! let result = notifications.next().await.unwrap().unwrap();
//! assert_eq!(*result.get(), 999);
//! # }
//! ```
//!
//! # Anti-Patterns
//!
//! ## ❌ Don't: Use `ordered_merge` When Order Doesn't Matter
//!
//! ```text
//! // BAD: Ordering overhead when you don't need it
//! let merged = stream1.ordered_merge(vec![stream2]);
//! ```
//!
//! Use standard futures combinators instead:
//!
//! ```text
//! // GOOD: Use futures::stream::select for unordered merging
//! use futures::stream::select;
//! let merged = select(stream1, stream2);
//! ```
//!
//! ## ❌ Don't: Use `combine_latest` for All Items
//!
//! ```text
//! // BAD: combine_latest only emits latest, loses intermediate values
//! let combined = stream1.combine_latest(vec![stream2], |_| true);
//! ```
//!
//! Use `ordered_merge` to get all items:
//!
//! ```text
//! // GOOD: ordered_merge emits every item
//! let merged = stream1.ordered_merge(vec![stream2]);
//! ```
//!
//! ## ❌ Don't: Complex Filter Logic in Operators
//!
//! ```text
//! // BAD: Complex business logic in filter predicate
//! stream1.combine_latest(vec![stream2], |state| {
//!     // 50 lines of complex filtering logic...
//! });
//! ```
//!
//! Extract to well-tested function:
//!
//! ```text
//! // GOOD: Testable, reusable filter logic
//! fn should_emit(state: &CombinedState<i32>) -> bool {
//!     // Clear, testable logic
//!     state.get_state().iter().all(|&v| v > 0)
//! }
//!
//! stream1.combine_latest(vec![stream2], should_emit);
//! ```
//!
//! # Operator Chaining
//!
//! Fluxion operators are designed to be composed together, creating sophisticated
//! data flows from simple building blocks. The key to successful chaining is
//! understanding how each operator transforms the stream.
//!
//! ## Basic Chaining Pattern
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, CombineWithPreviousExt, TakeLatestWhenExt};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (source_tx, source_rx) = mpsc::unbounded_channel();
//! let (filter_tx, filter_rx) = mpsc::unbounded_channel();
//!
//! let source_stream = UnboundedReceiverStream::new(source_rx);
//! let filter_stream = UnboundedReceiverStream::new(filter_rx);
//!
//! // Chain: sample when filter emits, unwrap the result, then pair with previous value
//! let mut composed = FluxionStream::new(source_stream)
//!     .take_latest_when(filter_stream, |_| true)
//!     .map(|item| item.unwrap())
//!     .combine_with_previous();
//!
//! filter_tx.send(Sequenced::new(1)).unwrap();
//! source_tx.send(Sequenced::new(42)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert!(item.previous.is_none());
//! assert_eq!(item.current.get(), &42);
//! }
//! ```
//!
//! ## Chaining with Transformation
//!
//! Use [`map_ordered`] and [`filter_ordered`] to transform streams while preserving
//! temporal ordering:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, Ordered};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx, rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let stream = UnboundedReceiverStream::new(rx);
//!
//! // Chain: filter positives, map to string
//! let mut composed = FluxionStream::new(stream)
//!     .filter_ordered(|n| *n > 0)
//!     .map_ordered(|stream_item| format!("Value: {}", stream_item.unwrap().get()));
//!
//! tx.send(Sequenced::new(-1)).unwrap();
//! tx.send(Sequenced::new(5)).unwrap();
//!
//! let result = composed.next().await.unwrap().unwrap();
//! assert_eq!(result, "Value: 5");
//! }
//! ```
//!
//! ## Multi-Stream Chaining
//!
//! Combine multiple streams and then process the result:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, CombineLatestExt, CombineWithPreviousExt, Ordered};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx1, rx1) = mpsc::unbounded_channel();
//! let (tx2, rx2) = mpsc::unbounded_channel();
//!
//! let stream1 = UnboundedReceiverStream::new(rx1);
//! let stream2 = UnboundedReceiverStream::new(rx2);
//!
//! // Chain: combine latest from both streams, then track changes
//! let mut composed = FluxionStream::new(stream1)
//!     .combine_latest(vec![stream2], |_| true)
//!     .combine_with_previous();
//!
//! tx1.send(Sequenced::new(1)).unwrap();
//! tx2.send(Sequenced::new(2)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert!(item.previous.is_none());
//! assert_eq!(item.current.get().get_state().len(), 2);
//! }
//! ```
//!
//! ## Key Principles for Chaining
//!
//! 1. **Use `map_ordered` and `filter_ordered`**: These preserve the `FluxionStream` wrapper
//!    and maintain temporal ordering guarantees
//! 2. **Order matters**: `combine_with_previous().filter_ordered()` is different from
//!    `filter_ordered().combine_with_previous()`
//! 3. **Type awareness**: Each operator changes the item type - track what flows through
//!    the chain
//! 4. **Test incrementally**: Build complex chains step by step, testing each addition
//!
//! ## Advanced Composition Examples
//!
//! ### 1. Ordered Merge → Combine With Previous
//!
//! Merge multiple streams in temporal order, then track consecutive values:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, OrderedStreamExt, CombineWithPreviousExt, Ordered};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<i32>>();
//!
//! let stream1 = UnboundedReceiverStream::new(rx1);
//! let stream2 = UnboundedReceiverStream::new(rx2);
//!
//! // Merge streams in temporal order, then pair consecutive values
//! let mut composed = FluxionStream::new(stream1)
//!     .ordered_merge(vec![FluxionStream::new(stream2)])
//!     .combine_with_previous();
//!
//! tx1.send(Sequenced::new(1)).unwrap();
//! let item = composed.next().await.unwrap().unwrap();
//! assert!(item.previous.is_none());
//! assert_eq!(item.current.get(), &1);
//!
//! tx2.send(Sequenced::new(2)).unwrap();
//! let item = composed.next().await.unwrap().unwrap();
//! assert_eq!(item.previous.unwrap().get(), &1);
//! assert_eq!(item.current.get(), &2);
//! }
//! ```
//!
//! ### 2. Combine Latest → Combine With Previous
//!
//! Combine latest values from multiple streams, then track state changes:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, CombineLatestExt, CombineWithPreviousExt, Ordered};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<i32>>();
//!
//! let stream1 = UnboundedReceiverStream::new(rx1);
//! let stream2 = UnboundedReceiverStream::new(rx2);
//!
//! // Combine latest, then track previous combined state
//! let mut composed = FluxionStream::new(stream1)
//!     .combine_latest(vec![stream2], |_| true)
//!     .combine_with_previous();
//!
//! tx1.send(Sequenced::new(1)).unwrap();
//! tx2.send(Sequenced::new(2)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert!(item.previous.is_none());
//! assert_eq!(item.current.get().get_state().len(), 2);
//!
//! tx1.send(Sequenced::new(3)).unwrap();
//! let item = composed.next().await.unwrap().unwrap();
//! // Previous state had [1, 2], current has [3, 2]
//! assert!(item.previous.is_some());
//! }
//! ```
//!
//! ### 3. Combine Latest → Take While With
//!
//! Combine streams and continue only while a condition holds:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, CombineLatestExt, TakeWhileExt};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();
//!
//! let stream1 = UnboundedReceiverStream::new(rx1);
//! let stream2 = UnboundedReceiverStream::new(rx2);
//! let filter_stream = UnboundedReceiverStream::new(filter_rx);
//!
//! // Combine latest values, but stop when filter becomes false
//! let mut composed = Box::pin(FluxionStream::new(stream1)
//!     .combine_latest(vec![stream2], |_| true)
//!     .take_while_with(filter_stream, |f| *f));
//!
//! filter_tx.send(Sequenced::new(true)).unwrap();
//! tx1.send(Sequenced::new(1)).unwrap();
//! tx2.send(Sequenced::new(2)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert_eq!(item.get_state().len(), 2);
//! }
//! ```
//!
//! ### 4. Ordered Merge → Take While With
//!
//! Merge streams in order and terminate based on external condition:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, OrderedStreamExt, TakeWhileExt};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<bool>>();
//!
//! let stream1 = UnboundedReceiverStream::new(rx1);
//! let stream2 = UnboundedReceiverStream::new(rx2);
//! let filter_stream = UnboundedReceiverStream::new(filter_rx);
//!
//! // Merge all values in order, but stop when filter says so
//! let mut composed = Box::pin(FluxionStream::new(stream1)
//!     .ordered_merge(vec![FluxionStream::new(stream2)])
//!     .take_while_with(filter_stream, |f| *f));
//!
//! filter_tx.send(Sequenced::new(true)).unwrap();
//! tx1.send(Sequenced::new(1)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert_eq!(item, 1);
//!
//! tx2.send(Sequenced::new(2)).unwrap();
//! let item = composed.next().await.unwrap().unwrap();
//! assert_eq!(item, 2);
//! }
//! ```
//!
//! ### 5. Take Latest When → Combine With Previous
//!
//! Sample latest value on trigger, then pair with previous sampled value:
//!
//! ```rust
//! use fluxion_stream::{FluxionStream, TakeLatestWhenExt, CombineWithPreviousExt, Ordered};
//! use fluxion_test_utils::Sequenced;
//! use tokio::sync::mpsc;
//! use tokio_stream::wrappers::UnboundedReceiverStream;
//! use futures::StreamExt;
//!
//! async fn example() {
//! let (source_tx, source_rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
//! let (filter_tx, filter_rx) = mpsc::unbounded_channel::<Sequenced<i32>>();
//!
//! let source_stream = UnboundedReceiverStream::new(source_rx);
//! let filter_stream = UnboundedReceiverStream::new(filter_rx);
//!
//! // Sample source when filter emits, unwrap the result, then track consecutive samples
//! let mut composed = FluxionStream::new(source_stream)
//!     .take_latest_when(filter_stream, |_| true)
//!     .map(|item| item.unwrap())
//!     .combine_with_previous();
//!
//! filter_tx.send(Sequenced::new(0)).unwrap();
//! source_tx.send(Sequenced::new(42)).unwrap();
//!
//! let item = composed.next().await.unwrap().unwrap();
//! assert!(item.previous.is_none());
//! assert_eq!(item.current.get(), &42);
//!
//! source_tx.send(Sequenced::new(99)).unwrap();
//! let item = composed.next().await.unwrap().unwrap();
//! assert_eq!(item.previous.unwrap().get(), &42);
//! assert_eq!(item.current.get(), &99);
//! }
//! ```
//!
//! These patterns demonstrate how Fluxion operators compose to create sophisticated
//! data flows. See the composition tests in the source repository for more examples.
//!
//! [`map_ordered`]: fluxion_stream::FluxionStream::map_ordered
//! [`filter_ordered`]: fluxion_stream::FluxionStream::filter_ordered
//!
//! # Getting Started
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fluxion-stream = { path = "../fluxion-stream" }
//! tokio = { version = "1.48", features = ["sync", "rt"] }
//! futures = "0.3"
//! ```
//!
//! See individual operator documentation for detailed examples.
//!
//! [`combine_latest`]: CombineLatestExt::combine_latest
//! [`with_latest_from`]: WithLatestFromExt::with_latest_from
//! [`ordered_merge`]: OrderedStreamExt::ordered_merge
//! [`emit_when`]: EmitWhenExt::emit_when
//! [`take_latest_when`]: TakeLatestWhenExt::take_latest_when
//! [`take_while_with`]: TakeWhileExt::take_while_with
//! [`combine_with_previous`]: CombineWithPreviousExt::combine_with_previous

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
#[macro_use]
mod logging;
pub mod combine_latest;
pub mod combine_with_previous;
pub mod emit_when;
pub mod fluxion_stream;
pub mod ordered_merge;
pub mod take_latest_when;
pub mod take_while_with;
pub mod types;
pub mod with_latest_from;

// Re-export commonly used types
pub use combine_latest::CombineLatestExt;
pub use combine_with_previous::CombineWithPreviousExt;
pub use emit_when::EmitWhenExt;
pub use fluxion_core::{Ordered, OrderedWrapper};
pub use fluxion_stream::FluxionStream;
pub use ordered_merge::OrderedStreamExt;
pub use take_latest_when::TakeLatestWhenExt;
pub use take_while_with::TakeWhileExt;
pub use types::{
    CombinedState, OrderedInner, OrderedInnerUnwrapped, OrderedStreamItem, WithPrevious,
};
pub use with_latest_from::WithLatestFromExt;
