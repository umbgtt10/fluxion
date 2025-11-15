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
//! let mut merged = Box::pin(merged);
//!
//! // Send out of order - stream2 sends seq=1, stream1 sends seq=2
//! tx2.send(Sequenced::with_sequence(100, 1)).unwrap();
//! tx1.send(Sequenced::with_sequence(200, 2)).unwrap();
//!
//! // Items are emitted in temporal order (seq 1, then seq 2)
//! let first = merged.next().await.unwrap();
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
//! # Common Patterns
//!
//! ## Pattern: Enriching Events with Context
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
//! let mut enriched = Box::pin(enriched);
//!
//! // Send config first, then click
//! config_tx.send(Sequenced::with_sequence("theme=dark".to_string(), 1)).unwrap();
//! click_tx.send(Sequenced::with_sequence("button1".to_string(), 2)).unwrap();
//!
//! let result = enriched.next().await.unwrap();
//! assert_eq!(result.get().len(), 2); // Has both click and config
//! # }
//! ```
//!
//! ## Pattern: Merging Multiple Event Sources
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
//! let mut unified_log = Box::pin(unified_log);
//!
//! // Send logs with different timestamps
//! service1_tx.send(Sequenced::with_sequence("service1: started".to_string(), 1)).unwrap();
//! service2_tx.send(Sequenced::with_sequence("service2: ready".to_string(), 2)).unwrap();
//!
//! let first = unified_log.next().await.unwrap();
//! assert_eq!(first.value, "service1: started");
//! # }
//! ```
//!
//! ## Pattern: Change Detection
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
//! let mut paired = Box::pin(paired);
//!
//! // Send values
//! tx.send(Sequenced::with_sequence(1, 1)).unwrap();
//! tx.send(Sequenced::with_sequence(1, 2)).unwrap(); // Same value
//! tx.send(Sequenced::with_sequence(2, 3)).unwrap(); // Changed!
//!
//! let result = paired.next().await.unwrap();
//! assert!(result.previous.is_none()); // First has no previous
//!
//! let result = paired.next().await.unwrap();
//! let changed = result.previous.as_ref().unwrap().value != result.current.value;
//! assert!(!changed); // 1 == 1, no change
//!
//! let result = paired.next().await.unwrap();
//! let changed = result.previous.as_ref().unwrap().value != result.current.value;
//! assert!(changed); // 1 != 2, changed!
//! # }
//! ```
//!
//! ## Pattern: Conditional Processing
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
//! let mut notifications = Box::pin(notifications);
//!
//! // Enable notifications
//! enabled_tx.send(Sequenced::with_sequence(1, 1)).unwrap();
//! // Send event
//! event_tx.send(Sequenced::with_sequence(999, 2)).unwrap();
//!
//! let result = notifications.next().await.unwrap();
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
pub mod with_latest_from;

// Re-export commonly used types
pub use combine_latest::{CombineLatestExt, CombinedState};
pub use combine_with_previous::CombineWithPreviousExt;
pub use emit_when::EmitWhenExt;
pub use fluxion_core::{Ordered, OrderedWrapper};
pub use fluxion_stream::FluxionStream;
pub use ordered_merge::OrderedStreamExt;
pub use take_latest_when::TakeLatestWhenExt;
pub use take_while_with::TakeWhileExt;
pub use with_latest_from::WithLatestFromExt;
