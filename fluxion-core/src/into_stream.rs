// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use futures::Stream;

/// A trait for types that can be converted into a `Stream`.
///
/// This allows stream operators to be more ergonomic by accepting
/// various stream-like types, such as channels or other wrappers,
/// and converting them into a stream internally.
///
/// # Examples
///
/// ```
/// use fluxion_core::IntoStream;
/// use futures::stream;
///
/// let values = vec![1, 2, 3];
/// let stream = stream::iter(values);
/// let converted = stream.into_stream();
/// ```
pub trait IntoStream {
    type Item;
    type Stream: Stream<Item = Self::Item>;
    fn into_stream(self) -> Self::Stream;
}

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
