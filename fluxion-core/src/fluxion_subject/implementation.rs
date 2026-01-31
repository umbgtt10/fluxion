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

        pub struct FluxionSubject<T: Clone + $($bounds)* 'static> {
            state: Arc<Mutex<SubjectState<T>>>,
        }

        impl<T: Clone + $($bounds)* 'static> FluxionSubject<T> {
            #[must_use]
            pub fn new() -> Self {
                Self {
                    state: Arc::new(Mutex::new(SubjectState {
                        closed: false,
                        senders: Vec::new(),
                    })),
                }
            }

            pub fn subscribe(&self) -> Result<SubjectBoxStream<T>, SubjectError> {
                let mut state = self.state.lock();
                if state.closed {
                    return Err(SubjectError::Closed);
                }

                let (tx, rx) = async_channel::unbounded();
                state.senders.push(tx);
                Ok(Box::pin(rx))
            }

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

            pub fn next(&self, value: T) -> Result<(), SubjectError> {
                self.send(StreamItem::Value(value))
            }

            pub fn error(&self, err: FluxionError) -> Result<(), SubjectError> {
                let result = self.send(StreamItem::Error(err));
                self.close();
                result
            }

            pub fn close(&self) {
                let mut state = self.state.lock();
                state.closed = true;
                state.senders.clear();
            }

            #[must_use]
            pub fn is_closed(&self) -> bool {
                self.state.lock().closed
            }

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
