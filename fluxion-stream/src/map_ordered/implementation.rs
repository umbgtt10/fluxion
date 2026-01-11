// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Common implementation for map_ordered operator.
//!
//! This module contains the shared logic used by both multi-threaded and single-threaded
//! versions of the map_ordered operator.

use fluxion_core::StreamItem;
use futures::{Stream, StreamExt};

/// Maps each item in a stream using the provided function.
///
/// This is the core implementation that transforms `StreamItem<T>` to `StreamItem<U>`
/// by applying the function to the contained value while preserving the stream structure.
#[inline]
pub(super) fn map_ordered_impl<S, T, U, F>(stream: S, mut f: F) -> impl Stream<Item = StreamItem<U>>
where
    S: Stream<Item = StreamItem<T>>,
    F: FnMut(T) -> U,
{
    stream.map(move |item| item.map(&mut f))
}
