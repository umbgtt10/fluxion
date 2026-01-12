// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Extension trait providing the `on_error` operator for streams.
///
/// This trait allows any stream of `StreamItem<T>` to handle errors
/// with a custom handler function.
///
/// Use [`OnErrorExt::on_error`] to use this operator.
///
/// # Behavior
///
/// - The handler receives a reference to each error and returns:
///     - `true` to consume the error (remove from stream)
///     - `false` to propagate the error downstream
///
/// Use `on_error` for side effects (logging) or error recovery (suppression).
///
/// # Examples
///
/// ## Basic Error Consumption
///
/// ```rust
/// use fluxion_stream::{OnErrorExt, IntoFluxionStream};
/// use fluxion_core::{FluxionError, StreamItem};
/// use fluxion_test_utils::Sequenced;
/// use futures::StreamExt;
///
/// # async fn example() {
/// let (tx, rx) = async_channel::unbounded();
/// let stream = rx.into_fluxion_stream();
///
/// let mut stream = stream
///     .on_error(|err| {
///         eprintln!("Error: {}", err);
///         true // Consume all errors
///     });
///
/// tx.try_send(Sequenced::new(1)).unwrap();
/// assert_eq!(stream.next().await.unwrap().unwrap().into_inner(), 1);
/// # }
/// ```
///
/// # See Also
///
/// - [`OnErrorExt::on_error`]
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
pub use multi_threaded::OnErrorExt;

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
pub use single_threaded::OnErrorExt;
