// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Extension trait providing the `scan_ordered` operator for streams.
//!
//! This operator accumulates state across stream items, emitting intermediate
//! accumulated values. It's similar to `Iterator::fold` but emits a result
//! for each input item rather than just the final result.
//!
//! # Key Characteristics
//!
//! - **Stateful**: Maintains accumulator state across all stream items
//! - **Emits per item**: Produces output for each input (unlike fold which emits once)
//! - **Type transformation**: Can change output type from input type
//! - **Error preserving**: Errors propagate without resetting state
//!
//! # Behavior
//!
//! - **Timestamp Preservation**: Output items preserve the timestamp of their source items
//! - **State Accumulation**: The accumulator state persists across all items
//! - **Error Handling**: Errors are propagated immediately without affecting accumulator state
//! - **Type Transformation**: Can transform input type to different output type
//!
//! # Examples
//!
//! ## Running Sum
//!
//! ```rust
//! use fluxion_stream::ScanOrderedExt;
//! use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! // Accumulate sum
//! let mut sums = stream.scan_ordered::<Sequenced<i32>, _, _>(0, |acc, val| {
//!     *acc += val;
//!     *acc
//! });
//!
//! tx.unbounded_send((10, 1).into())?;
//! tx.unbounded_send((20, 2).into())?;
//! tx.unbounded_send((30, 3).into())?;
//!
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 10);  // 0 + 10
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 30);  // 10 + 20
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut sums, 500).await)).value, 60);  // 30 + 30
//! # Ok(())
//! # }
//! ```
//!
//! ## Type Transformation: Count to String
//!
//! ```rust
//! use fluxion_stream::ScanOrderedExt;
//! use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let (tx, stream) = test_channel::<Sequenced<String>>();
//!
//! // Count items and produce formatted strings
//! let mut counts = stream.scan_ordered::<Sequenced<String>, _, _>(0i32, |count: &mut i32, _item: &String| {
//!     *count += 1;
//!     format!("Item #{}", count)
//! });
//!
//! tx.unbounded_send(("apple".to_string(), 1).into())?;
//! tx.unbounded_send(("banana".to_string(), 2).into())?;
//!
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut counts, 500).await)).value, "Item #1");
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut counts, 500).await)).value, "Item #2");
//! # Ok(())
//! # }
//! ```
//!
//! ## Complex State: Building a List
//!
//! ```rust
//! use fluxion_stream::ScanOrderedExt;
//! use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream, unwrap_value};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let (tx, stream) = test_channel::<Sequenced<i32>>();
//!
//! // Collect all values seen so far
//! let mut history = stream.scan_ordered::<Sequenced<Vec<i32>>, _, _>(Vec::<i32>::new(), |list: &mut Vec<i32>, val: &i32| {
//!     list.push(*val);
//!     list.clone() // Return snapshot
//! });
//!
//! tx.unbounded_send((10, 1).into())?;
//! tx.unbounded_send((20, 2).into())?;
//! tx.unbounded_send((30, 3).into())?;
//!
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10]);
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10, 20]);
//! assert_eq!(unwrap_value(Some(unwrap_stream(&mut history, 500).await)).value, vec![10, 20, 30]);
//! # Ok(())
//! # }
//! ```
//!
//! # Use Cases
//!
//! - Running totals, averages, or statistics
//! - Building collections or aggregations over time
//! - Counting occurrences or tracking frequencies
//! - State machines with accumulated context
//! - Moving window calculations
//!
//! # Advanced Example: State Machine
//!
//! ```rust
//! use fluxion_stream::ScanOrderedExt;
//! use fluxion_test_utils::{Sequenced, test_channel, unwrap_stream};
//!
//! #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! enum Event { Login, Logout, Action(String) }
//!
//! #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
//! struct SessionState {
//!     logged_in: bool,
//!     action_count: usize,
//! }
//!
//! # async fn example() -> anyhow::Result<()> {
//! let (tx, stream) = test_channel::<Sequenced<Event>>();
//!
//! let mut states = stream.scan_ordered::<Sequenced<SessionState>, _, _>(
//!     SessionState { logged_in: false, action_count: 0 },
//!     |state, event| {
//!         match event {
//!             Event::Login => state.logged_in = true,
//!             Event::Logout => {
//!                 state.logged_in = false;
//!                 state.action_count = 0;
//!             }
//!             Event::Action(_) if state.logged_in => {
//!                 state.action_count += 1;
//!             }
//!             _ => {}
//!         }
//!         state.clone()
//!     }
//! );
//!
//! tx.unbounded_send(Sequenced::new(Event::Login))?;
//! let s1 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
//! assert!(s1.logged_in && s1.action_count == 0);
//!
//! tx.unbounded_send(Sequenced::new(Event::Action("click".to_string())))?;
//! let s2 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
//! assert!(s2.logged_in && s2.action_count == 1);
//!
//! tx.unbounded_send(Sequenced::new(Event::Logout))?;
//! let s3 = unwrap_stream(&mut states, 500).await.unwrap().into_inner();
//! assert!(!s3.logged_in && s3.action_count == 0);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - O(1) time per item (depends on accumulator function)
//! - Stores only the accumulator state (not all items)
//! - Minimal overhead - just function calls and locking
#[macro_use]
mod implementation;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
mod multi_threaded;

#[cfg(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
))]
pub use multi_threaded::ScanOrderedExt;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
mod single_threaded;

#[cfg(not(any(
    all(feature = "runtime-tokio", not(target_arch = "wasm32")),
    feature = "runtime-smol",
    feature = "runtime-async-std"
)))]
pub use single_threaded::ScanOrderedExt;
