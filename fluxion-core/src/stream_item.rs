// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::error::FluxionError;

/// A stream item that can be either a value or an error.
///
/// This enum allows operators to naturally propagate errors through the stream
/// while processing values, following Rx-style error semantics where errors
/// terminate the sequence.
#[derive(Debug, Clone)]
pub enum StreamItem<T> {
    /// A successful value
    Value(T),
    /// An error that terminates the stream
    Error(FluxionError),
}

impl<T: PartialEq> PartialEq for StreamItem<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StreamItem::Value(a), StreamItem::Value(b)) => a == b,
            _ => false, // Errors are never equal
        }
    }
}

impl<T: Eq> Eq for StreamItem<T> {}

impl<T: PartialOrd> PartialOrd for StreamItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (StreamItem::Value(a), StreamItem::Value(b)) => a.partial_cmp(b),
            (StreamItem::Error(_), StreamItem::Error(_)) => Some(std::cmp::Ordering::Equal),
            (StreamItem::Error(_), StreamItem::Value(_)) => Some(std::cmp::Ordering::Less),
            (StreamItem::Value(_), StreamItem::Error(_)) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl<T: Ord> Ord for StreamItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (StreamItem::Value(a), StreamItem::Value(b)) => a.cmp(b),
            (StreamItem::Error(_), StreamItem::Error(_)) => std::cmp::Ordering::Equal,
            (StreamItem::Error(_), StreamItem::Value(_)) => std::cmp::Ordering::Less,
            (StreamItem::Value(_), StreamItem::Error(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl<T> StreamItem<T> {
    /// Returns `true` if this is a `Value`.
    pub const fn is_value(&self) -> bool {
        matches!(self, StreamItem::Value(_))
    }

    /// Returns `true` if this is an `Error`.
    pub const fn is_error(&self) -> bool {
        matches!(self, StreamItem::Error(_))
    }

    /// Converts from `StreamItem<T>` to `Option<T>`, discarding errors.
    pub fn ok(self) -> Option<T> {
        match self {
            StreamItem::Value(v) => Some(v),
            StreamItem::Error(_) => None,
        }
    }

    /// Converts from `StreamItem<T>` to `Option<FluxionError>`, discarding values.
    pub fn err(self) -> Option<FluxionError> {
        match self {
            StreamItem::Value(_) => None,
            StreamItem::Error(e) => Some(e),
        }
    }

    /// Maps a `StreamItem<T>` to `StreamItem<U>` by applying a function to the contained value.
    ///
    /// Errors are propagated unchanged.
    pub fn map<U, F>(self, f: F) -> StreamItem<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            StreamItem::Value(v) => StreamItem::Value(f(v)),
            StreamItem::Error(e) => StreamItem::Error(e),
        }
    }

    /// Maps a `StreamItem<T>` to `StreamItem<U>` by applying a function that can fail.
    ///
    /// Errors are propagated unchanged.
    pub fn and_then<U, F>(self, f: F) -> StreamItem<U>
    where
        F: FnOnce(T) -> StreamItem<U>,
    {
        match self {
            StreamItem::Value(v) => f(v),
            StreamItem::Error(e) => StreamItem::Error(e),
        }
    }

    /// Returns the contained value, panicking if it's an error.
    ///
    /// # Panics
    ///
    /// Panics if the item is an `Error`.
    pub fn unwrap(self) -> T
    where
        FluxionError: std::fmt::Debug,
    {
        match self {
            StreamItem::Value(v) => v,
            StreamItem::Error(e) => {
                panic!("called `StreamItem::unwrap()` on an `Error` value: {:?}", e)
            }
        }
    }

    /// Returns the contained value, panicking with a custom message if it's an error.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the item is an `Error`.
    pub fn expect(self, msg: &str) -> T
    where
        FluxionError: std::fmt::Debug,
    {
        match self {
            StreamItem::Value(v) => v,
            StreamItem::Error(e) => panic!("{}: {:?}", msg, e),
        }
    }
}

impl<T> From<Result<T, FluxionError>> for StreamItem<T> {
    fn from(result: Result<T, FluxionError>) -> Self {
        match result {
            Ok(v) => StreamItem::Value(v),
            Err(e) => StreamItem::Error(e),
        }
    }
}

impl<T> From<StreamItem<T>> for Result<T, FluxionError> {
    fn from(item: StreamItem<T>) -> Self {
        match item {
            StreamItem::Value(v) => Ok(v),
            StreamItem::Error(e) => Err(e),
        }
    }
}

impl<T> crate::HasTimestamp for StreamItem<T>
where
    T: crate::Timestamped,
{
    type Inner = T::Inner;
    type Timestamp = T::Timestamp;

    fn timestamp(&self) -> Self::Timestamp {
        match self {
            StreamItem::Value(v) => v.timestamp(),
            StreamItem::Error(_) => panic!("called `timestamp()` on StreamItem::Error"),
        }
    }
}

impl<T> crate::Timestamped for StreamItem<T>
where
    T: crate::Timestamped,
{
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        StreamItem::Value(T::with_timestamp(value, timestamp))
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        StreamItem::Value(T::with_fresh_timestamp(value))
    }

    fn into_inner(self) -> Self::Inner {
        match self {
            StreamItem::Value(v) => v.into_inner(),
            StreamItem::Error(_) => panic!("called `into_inner()` on StreamItem::Error"),
        }
    }
}
