// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::StreamItem;
use alloc::vec::Vec;
use async_channel::Sender;

pub(crate) struct SubjectState<T> {
    pub(crate) closed: bool,
    pub(crate) senders: Vec<Sender<StreamItem<T>>>,
}

macro_rules! define_subject_impl {
    ($($bounds:tt)*) => {
        use crate::fluxion_mutex::Mutex;
        use crate::{FluxionError, StreamItem, SubjectError};
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::vec::Vec;
        use core::pin::Pin;
        use futures::stream::Stream;
        use crate::fluxion_subject::implementation::SubjectState;

        type SubjectBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + $($bounds)* 'static>>;

        /// A hot, unbounded subject that broadcasts items to all current subscribers.
        ///
        /// `FluxionSubject` is the entry point for pushing values into a Fluxion stream pipeline.
        /// It implements a publish-subscribe pattern where multiple subscribers can receive
        /// the same items.
        ///
        /// See the [module documentation](crate::fluxion_subject) for examples and more details.
        pub struct FluxionSubject<T: Clone + $($bounds)* 'static> {
            state: Arc<Mutex<SubjectState<T>>>,
        }

        impl<T: Clone + $($bounds)* 'static> FluxionSubject<T> {
            /// Creates a new unbounded subject with no subscribers.
            ///
            /// The subject starts in an open state and can immediately accept
            /// subscriptions and items.
            #[must_use]
            pub fn new() -> Self {
                Self {
                    state: Arc::new(Mutex::new(SubjectState {
                        closed: false,
                        senders: Vec::new(),
                    })),
                }
            }

            /// Subscribe to this subject and receive a stream of `StreamItem<T>`.
            /// Late subscribers do not receive previously sent items.
            pub fn subscribe(&self) -> Result<SubjectBoxStream<T>, SubjectError> {
                let mut state = self.state.lock();
                if state.closed {
                    return Err(SubjectError::Closed);
                }

                let (tx, rx) = async_channel::unbounded();
                state.senders.push(tx);
                Ok(Box::pin(rx))
            }

            /// Send an item to all active subscribers.
            ///
            /// Returns a subject-specific error if the operation fails:
            /// - `SubjectError::Closed` if the subject has been closed
            pub fn send(&self, item: StreamItem<T>) -> Result<(), SubjectError> {
                let mut state = self.state.lock();
                if state.closed {
                    return Err(SubjectError::Closed);
                }

                let mut next_senders = Vec::with_capacity(state.senders.len());

                for tx in state.senders.drain(..) {
                    if tx.try_send(item.clone()).is_ok() {
                        next_senders.push(tx);
                    }
                }

                state.senders = next_senders;
                Ok(())
            }

            /// Send a value to all active subscribers.
            ///
            /// This is a convenience wrapper around `send(StreamItem::Value(value))`.
            ///
            /// # Errors
            ///
            /// Returns `SubjectError::Closed` if the subject has been closed.
            pub fn next(&self, value: T) -> Result<(), SubjectError> {
                self.send(StreamItem::Value(value))
            }

            /// Convenience helper to send a stream error to all subscribers and terminate the subject.
            ///
            /// This bridges stream errors (FluxionError) with subject operations (SubjectError).
            pub fn error(&self, err: FluxionError) -> Result<(), SubjectError> {
                let result = self.send(StreamItem::Error(err));
                self.close();
                result
            }

            /// Closes the subject, completing all subscriber streams.
            ///
            /// After closing:
            /// - All existing subscribers will receive `None` on their next poll (stream ends).
            /// - `send()` and `error()` will return `SubjectError::Closed`.
            /// - `subscribe()` will return `SubjectError::Closed`.
            ///
            /// Closing is idempotent—calling it multiple times has no additional effect.
            pub fn close(&self) {
                let mut state = self.state.lock();
                state.closed = true;
                state.senders.clear();
            }

            /// Returns `true` if the subject has been closed.
            ///
            /// A closed subject cannot accept new items or subscribers.
            #[must_use]
            pub fn is_closed(&self) -> bool {
                self.state.lock().closed
            }

            /// Returns the number of currently active subscribers.
            ///
            /// Note: This count is updated lazily—dropped subscribers are removed
            /// on the next `send()` call, not immediately when dropped.
            #[must_use]
            pub fn subscriber_count(&self) -> usize {
                self.state.lock().senders.len()
            }
        }

        impl<T: Clone + $($bounds)* 'static> Default for FluxionSubject<T> {
            fn default() -> Self {
                Self::new()
            }
        }

        impl<T: Clone + $($bounds)* 'static> Clone for FluxionSubject<T> {
            fn clone(&self) -> Self {
                Self {
                    state: self.state.clone(),
                }
            }
        }
    };
}
