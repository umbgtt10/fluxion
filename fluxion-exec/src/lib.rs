// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg_attr(
    not(any(
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    )),
    no_std
)]
#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]

//! Async execution utilities for stream processing.
//!
//! This crate provides subscription-based execution patterns for consuming streams
//! with async handlers. It focuses on the **execution** of stream processing, while
//! `fluxion-stream` focuses on **composition** of streams.
//!
//! # Overview
//!
//! The execution utilities solve a common problem: how to process stream items with
//! async functions while controlling concurrency and cancellation behavior.
//!
//! ## Key Concepts
//!
//! - **Subscription**: Attach an async handler to a stream and run it to completion
//! - **Sequential execution**: Process items one at a time (no concurrent handlers)
//! - **Cancellation**: Automatically cancel outdated work when new items arrive
//! - **Error handling**: Propagate errors from handlers while continuing stream processing
//!
//! # Execution Patterns
//!
//! This crate provides two execution patterns:
//!
//! ## [`subscribe`] - Sequential Processing
//!
//! Process each item sequentially with an async handler. Every item is processed
//! to completion before the next item is handled.
//!
//! **Use when:**
//! - Every item must be processed
//! - Processing order matters
//! - Side effects must occur for each item
//! - Work cannot be skipped
//!
//! **Examples:**
//! - Writing each event to a database
//! - Sending each notification
//! - Processing every transaction
//! - Logging all events
//!
//! ## [`subscribe_latest`] - Latest-Value Processing
//!
//! Process only the latest item, automatically canceling work for outdated items.
//! When a new item arrives while processing, the current work is canceled and the
//! new item is processed instead.
//!
//! **Use when:**
//! - Only the latest value matters
//! - Old values become irrelevant
//! - Expensive operations should skip intermediate values
//! - UI updates or state synchronization
//!
//! **Examples:**
//! - Rendering UI based on latest state
//! - Auto-saving the current document
//! - Updating a preview
//! - Recalculating derived values
//!
//! # Architecture
//!
//! ## Extension Trait Pattern
//!
//! Both utilities are provided as extension traits on `Stream`:
//!
//! ```text
//! use fluxion_exec::SubscribeExt;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//! let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
//!
//! // Any Stream can use subscribe
//! stream.subscribe(|value| async move {
//!     println!("Processing: {}", value);
//!     Ok::<_, Box<dyn std::error::Error>>(())
//! }).await;
//! # }
//! ```
//!
//! ## Task Spawning
//!
//! Both patterns spawn tokio tasks internally:
//!
//! - **[`subscribe`]**: Spawns one task per stream item (sequential)
//! - **[`subscribe_latest`]**: Spawns tasks and cancels obsolete ones
//!
//! This means:
//! - Handlers must be `Send + 'static`
//! - Processing happens on the tokio runtime
//! - Multiple streams can be processed concurrently
//!
//! # Performance Characteristics
//!
//! ## Sequential Processing (`subscribe`)
//!
//! - **Latency**: Items wait for previous items to complete
//! - **Throughput**: Limited by handler execution time
//! - **Memory**: $O(1)$ - processes one item at a time
//! - **Ordering**: Maintains strict order
//!
//! **Best for**: Correctness over throughput
//!
//! ## Latest-Value Processing (`subscribe_latest`)
//!
//! - **Latency**: Immediate start on new items (cancels old work)
//! - **Throughput**: Skips intermediate values for efficiency
//! - **Memory**: $O(1)$ - one active task at a time
//! - **Ordering**: Processes latest available
//!
//! **Best for**: Responsiveness over completeness
//!
//! # Comparison with Other Patterns
//!
//! ## vs `for_each` (futures)
//!
//! ```text
//! // futures::StreamExt::for_each - blocks until stream ends
//! stream.for_each(|item| async {
//!     process(item).await;
//! }).await;
//!
//! // subscribe - returns immediately, spawns background task
//! stream.subscribe(process).await;
//! ```
//!
//! ## vs `buffer_unordered` (futures)
//!
//! ```text
//! // futures - processes N items concurrently
//! stream.map(process).buffer_unordered(10).collect().await;
//!
//! // subscribe - strictly sequential
//! stream.subscribe(process).await;
//! ```
//!
//! ## vs Manual Task Spawning
//!
//! ```text
//! // Manual - no cancellation on new items
//! while let Some(item) = stream.next().await {
//!     tokio::spawn(async move { process(item).await });
//! }
//!
//! // subscribe_latest - automatic cancellation
//! stream.subscribe_latest(process).await;
//! ```
//!
//! # Common Patterns
//!
//! ## Pattern: Database Writes
//!
//! Every item must be persisted:
//!
//! ```text
//! use fluxion_exec::SubscribeExt;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! # let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//! # let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
//! stream.subscribe(|event| async move {
//!     // Save to database
//!     // database.insert(event).await?;
//!     Ok::<_, Box<dyn std::error::Error>>(())
//! }).await;
//! # }
//! ```
//!
//! ## Pattern: UI Updates
//!
//! Only latest state matters:
//!
//! ```text
//! use fluxion_exec::SubscribeLatestExt;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! # let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//! # let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
//! stream.subscribe_latest(|state| async move {
//!     // Render UI with latest state
//!     // update_ui(state).await?;
//!     Ok::<_, Box<dyn std::error::Error>>(())
//! }).await;
//! # }
//! ```
//!
//! ## Pattern: Batch Processing
//!
//! Combine with `chunks` for batch operations:
//!
//! ```text
//! use fluxion_exec::SubscribeExt;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! # let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//! # let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
//! stream
//!     .chunks(100)  // Batch 100 items
//!     .subscribe(|batch| async move {
//!         // Process batch
//!         // database.insert_batch(batch).await?;
//!         Ok::<_, Box<dyn std::error::Error>>(())
//!     })
//!     .await;
//! # }
//! ```
//!
//! ## Pattern: Error Recovery
//!
//! Handle errors without stopping the stream:
//!
//! ```text
//! use fluxion_exec::SubscribeExt;
//! use futures::StreamExt;
//!
//! # async fn example() {
//! # let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
//! # let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
//! stream.subscribe(|item| async move {
//!     match process_item(item).await {
//!         Ok(result) => Ok(()),
//!         Err(e) => {
//!             eprintln!("Error processing item: {}", e);
//!             Ok(())  // Continue processing despite error
//!         }
//!     }
//! }).await;
//!
//! # async fn process_item(_item: i32) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//! # }
//! ```
//!
//! # Anti-Patterns
//!
//! ## ❌ Don't: Use `subscribe_latest` for Critical Work
//!
//! ```text
//! // BAD: Payment processing might be skipped!
//! payment_stream.subscribe_latest(|payment| async move {
//!     process_payment(payment).await  // Could be canceled!
//! }).await;
//! ```
//!
//! Use `subscribe` for work that must complete:
//!
//! ```text
//! // GOOD: Every payment is processed
//! payment_stream.subscribe(|payment| async move {
//!     process_payment(payment).await
//! }).await;
//! ```
//!
//! ## ❌ Don't: Block in Handlers
//!
//! ```text
//! // BAD: Blocking operations stall the executor
//! stream.subscribe(|item| async move {
//!     std::thread::sleep(Duration::from_secs(1));  // Blocks!
//!     Ok(())
//! }).await;
//! ```
//!
//! Use async operations or `spawn_blocking`:
//!
//! ```text
//! // GOOD: Async sleep doesn't block
//! stream.subscribe(|item| async move {
//!     tokio::time::sleep(Duration::from_secs(1)).await;
//!     Ok(())
//! }).await;
//! ```
//!
//! ## ❌ Don't: Use for CPU-Intensive Work
//!
//! ```text
//! // BAD: CPU-intensive work on async runtime
//! stream.subscribe(|data| async move {
//!     expensive_computation(data);  // Blocks executor!
//!     Ok(())
//! }).await;
//! ```
//!
//! Offload to blocking threadpool:
//!
//! ```text
//! // GOOD: CPU work on dedicated threads
//! stream.subscribe(|data| async move {
//!     tokio::task::spawn_blocking(move || {
//!         expensive_computation(data)
//!     }).await?;
//!     Ok(())
//! }).await;
//! ```
//!
//! # Error Handling
//!
//! Both subscription methods return `Result`:
//!
//! - **`Ok(())`**: Stream completed successfully
//! - **`Err(e)`**: Handler returned an error
//!
//! Errors from handlers are propagated but don't stop stream processing automatically.
//! Design your handlers to return `Ok(())` to continue processing despite errors, or
//! return `Err(e)` to stop on first error.
//!
//! # Getting Started
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fluxion-exec = { path = "../fluxion-exec" }
//! tokio = { version = "1.48", features = ["rt", "sync"] }
//! futures = "0.3"
//! ```
//!
//! See individual trait documentation for detailed examples:
//! - [`SubscribeExt`] for sequential processing
//! - [`SubscribeLatestExt`] for latest-value processing
//!
//! [`subscribe`]: SubscribeExt::subscribe
//! [`subscribe_latest`]: SubscribeLatestExt::subscribe_latest

extern crate alloc;

#[macro_use]
mod logging;
pub mod subscribe;
#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-smol",
    feature = "runtime-async-std",
    target_arch = "wasm32"
))]
pub mod subscribe_latest;

// Re-export commonly used types
pub use subscribe::SubscribeExt;
#[cfg(any(
    feature = "runtime-tokio",
    feature = "runtime-smol",
    feature = "runtime-async-std",
    target_arch = "wasm32"
))]
pub use subscribe_latest::SubscribeLatestExt;

/// Helper function to ignore errors in subscribe callbacks.
///
/// This provides a convenient way to explicitly ignore errors when using
/// [`SubscribeExt::subscribe`] or [`SubscribeLatestExt::subscribe_latest`].
///
/// # Examples
///
/// ```
/// use fluxion_exec::{SubscribeExt, ignore_errors};
/// use futures::stream;
///
/// # #[tokio::main]
/// # async fn main() {
/// let stream = stream::iter(vec![1, 2, 3]);
///
/// stream.subscribe(
///     |item, _token| async move {
///         println!("Processing: {}", item);
///         Ok::<(), std::io::Error>(())
///     },
///     None,
///     ignore_errors  // Explicitly ignore errors
/// ).await.unwrap();
/// # }
/// ```
#[inline]
pub fn ignore_errors<E>(_error: E) {
    // Intentionally empty - discards the error
}
