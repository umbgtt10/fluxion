// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Generic implementation of share operator - runtime-agnostic

use alloc::boxed::Box;
use core::pin::Pin;
use fluxion_core::{FluxionSubject, FluxionTask, StreamItem, SubjectError};
use futures::{Stream, StreamExt};

pub type SharedBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>>;

/// Generic implementation of FluxionShared - works with any runtime that supports FluxionTask
pub struct FluxionShared<T: Clone + Send + Sync + 'static> {
    subject: FluxionSubject<T>,
    _task: FluxionTask,
}

impl<T: Clone + Send + Sync + 'static> FluxionShared<T> {
    /// Creates a new `FluxionShared` from a source stream.
    ///
    /// Spawns a task that consumes the source and forwards items to subscribers.
    pub fn new<S>(source: S) -> Self
    where
        S: Stream<Item = StreamItem<T>> + Send + Unpin + 'static,
    {
        let subject = FluxionSubject::new();
        let subject_clone = subject.clone();

        let task = FluxionTask::spawn(|cancel| async move {
            let mut stream = source;
            loop {
                if cancel.is_cancelled() {
                    break;
                }

                match stream.next().await {
                    Some(StreamItem::Value(v)) => {
                        if subject_clone.next(v).is_err() {
                            break;
                        }
                    }
                    Some(StreamItem::Error(e)) => {
                        let _ = subject_clone.error(e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
            subject_clone.close();
        });

        Self {
            subject,
            _task: task,
        }
    }

    /// Subscribe to this shared source and receive a stream of items.
    ///
    /// Late subscribers do not receive previously emitted items.
    pub fn subscribe(&self) -> Result<SharedBoxStream<T>, SubjectError> {
        // FluxionSubject returns Send+Sync boxed stream
        self.subject.subscribe().map(|stream| {
            let stream: Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync + 'static>> = stream;
            stream
        })
    }

    /// Returns `true` if the shared source has completed or errored.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.subject.is_closed()
    }

    /// Returns the number of currently active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subject.subscriber_count()
    }
}

impl<T: Clone + Send + Sync + 'static> Drop for FluxionShared<T> {
    fn drop(&mut self) {
        self.subject.close();
    }
}
