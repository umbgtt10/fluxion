// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use futures::Stream;

/// A trait for types that can be converted into a `Stream`.
///
/// This allows stream operators to be more ergonomic by accepting
/// various stream-like types, such as channels or other wrappers,
/// and converting them into a stream internally.
pub trait IntoStream {
    /// The type of items in the stream.
    type Item;
    /// The stream type that this object can be converted into.
    type Stream: Stream<Item = Self::Item>;

    /// Converts this object into a stream.
    fn into_stream(self) -> Self::Stream;
}

/// Blanket implementation for any type that is already a `Stream`.
/// This allows functions that accept `IntoStream` to also accept any `Stream`
/// without needing explicit conversion.
impl<S> IntoStream for S
where
    S: Stream,
{
    type Item = S::Item;
    type Stream = S;

    fn into_stream(self) -> Self::Stream {
        self
    }
}
