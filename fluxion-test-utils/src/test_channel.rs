// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Test channel infrastructure with push capabilities.
//!
//! `TestChannel` provides a clean testing API that separates mutation (push)
//! from stream consumption (into_stream). This solves the fundamental conflict
//! between imperative test setup and functional stream operations.

use crate::fluxion_channel::FluxionChannel;
use crate::sequenced::Sequenced;
use fluxion_error::Result;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// A test-oriented channel that provides both push capabilities and stream conversion.
///
/// # Design Philosophy
///
/// `TestChannel` is designed for **test code only**. It wraps a sender and stream, providing:
/// - **Mutation**: `push()` method to add values imperatively during test setup
/// - **Consumption**: Generic stream type for functional operations
///
/// This separation allows tests to:
/// 1. Set up channels with `TestChannel::new()`
/// 2. Push initial values with `channel.push(value)`
/// 3. Use the stream field directly or convert via From trait
///
/// For compatibility with existing tests, `sender` and `stream` fields are public.
///
/// # Example
///
/// ```rust,ignore
/// use fluxion_test_utils::TestChannel;
/// use fluxion::FluxionStream;
///
/// #[tokio::test]
/// async fn test_stream_operations() {
///     let mut person = TestChannel::new();
///     let mut animal = TestChannel::new();
///
///     // Imperative setup
///     person.push(Person { id: 1, name: "Alice" });
///     animal.push(Animal { id: 1, species: "Cat" });
///
///     // Convert to functional streams
///     let person_stream = FluxionStream::new(person.stream);
///     let animal_stream = FluxionStream::new(animal.stream);
///
///     // Use consuming operations
///     let combined = person_stream.combine_latest(
///         vec![animal_stream],
///         |state| state.all_have_values()
///     );
/// }
/// ```
pub struct TestChannel<T, S = UnboundedReceiverStream<Sequenced<T>>> {
    pub sender: crate::fluxion_channel::sequenced_channel::UnboundedSender<T>,
    pub stream: S,
}

impl<T> TestChannel<T, UnboundedReceiverStream<Sequenced<T>>> {
    /// Creates a new test channel.
    #[must_use]
    pub fn new() -> Self {
        let channel = FluxionChannel::new();
        Self {
            sender: channel.sender,
            stream: channel.stream,
        }
    }

    /// Creates an empty test channel (sender already dropped).
    #[must_use]
    pub fn empty() -> Self {
        let channel = FluxionChannel::empty();
        Self {
            sender: channel.sender,
            stream: channel.stream,
        }
    }

    /// Push a value into the channel, returning an error if the receiver is dropped.
    ///
    /// # Errors
    /// Returns `Err(FluxionError::ChannelSendError)` if the receiver has been dropped.
    pub fn push(&self, value: T) -> Result<()> {
        self.sender
            .send(value)
            .map_err(|_| fluxion_error::FluxionError::ChannelSendError)
    }

    /// Push a value into the channel, panicking if the receiver is dropped.
    ///
    /// This is provided for backward compatibility with existing tests.
    /// New code should prefer `push()` and handle the Result.
    ///
    /// # Panics
    /// Panics if the receiver has been dropped.
    pub fn push_unchecked(&self, value: T) {
        self.sender.send(value).expect("receiver dropped");
    }

    /// Close the channel by dropping the sender.
    pub fn close(self) {
        drop(self.sender);
    }

    /// Convert this test channel into the underlying stream.
    ///
    /// This consumes the `TestChannel` and returns the stream.
    /// You can wrap it in `FluxionStream::new()` for extension methods.
    #[must_use]
    pub fn into_stream(self) -> UnboundedReceiverStream<Sequenced<T>> {
        self.stream
    }
}

impl<T> Default for TestChannel<T, UnboundedReceiverStream<Sequenced<T>>> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create multiple test channels at once.
pub struct TestChannels;

impl TestChannels {
    /// Creates three test channels.
    #[must_use]
    pub fn three<T>() -> (TestChannel<T>, TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new(), TestChannel::new())
    }

    /// Creates two test channels.
    #[must_use]
    pub fn two<T>() -> (TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        id: u32,
        value: String,
    }

    #[tokio::test]
    async fn test_channel_push_and_convert() {
        let channel = TestChannel::new();

        channel
            .push(TestData {
                id: 1,
                value: "Alice".to_string(),
            })
            .unwrap();
        channel
            .push(TestData {
                id: 2,
                value: "Bob".to_string(),
            })
            .unwrap();

        let mut stream = channel.into_stream();

        let item1 = stream.next().await.unwrap();
        assert_eq!(item1.value.id, 1);
        assert_eq!(item1.value.value, "Alice");

        let item2 = stream.next().await.unwrap();
        assert_eq!(item2.value.id, 2);
        assert_eq!(item2.value.value, "Bob");
    }

    #[tokio::test]
    async fn test_channel_close() {
        let channel = TestChannel::new();

        channel
            .push(TestData {
                id: 1,
                value: "Test".to_string(),
            })
            .unwrap();
        channel.close();

        // Cannot push after close since channel is consumed
    }

    #[tokio::test]
    async fn test_channels_helper_two() {
        let (ch1, ch2) = TestChannels::two::<TestData>();

        ch1.push(TestData {
            id: 1,
            value: "A".to_string(),
        })
        .unwrap();
        ch2.push(TestData {
            id: 2,
            value: "B".to_string(),
        })
        .unwrap();

        let mut s1 = ch1.into_stream();
        let mut s2 = ch2.into_stream();

        assert_eq!(s1.next().await.unwrap().value.id, 1);
        assert_eq!(s2.next().await.unwrap().value.id, 2);
    }

    #[tokio::test]
    async fn test_channels_helper_three() {
        let (ch1, ch2, ch3) = TestChannels::three::<TestData>();

        ch1.push(TestData {
            id: 1,
            value: "A".to_string(),
        })
        .unwrap();
        ch2.push(TestData {
            id: 2,
            value: "B".to_string(),
        })
        .unwrap();
        ch3.push(TestData {
            id: 3,
            value: "C".to_string(),
        })
        .unwrap();

        let mut s1 = ch1.into_stream();
        let mut s2 = ch2.into_stream();
        let mut s3 = ch3.into_stream();

        assert_eq!(s1.next().await.unwrap().value.id, 1);
        assert_eq!(s2.next().await.unwrap().value.id, 2);
        assert_eq!(s3.next().await.unwrap().value.id, 3);
    }
}
