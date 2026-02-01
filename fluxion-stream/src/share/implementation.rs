// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_share_impl {
    ($($bounds:tt)*) => {
        use alloc::boxed::Box;
        use core::pin::Pin;
        use fluxion_core::{FluxionSubject, FluxionTask, StreamItem, SubjectError};
        use futures::{
            future::{select, Either},
            Stream, StreamExt,
        };

        pub type SharedBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> + $($bounds)* 'static>>;

        pub struct FluxionShared<T: Clone + $($bounds)* 'static> {
            subject: FluxionSubject<T>,
            _task: FluxionTask,
        }

        impl<T: Clone + $($bounds)* 'static> FluxionShared<T> {
            pub fn new<S>(source: S) -> Self
            where
                S: Stream<Item = StreamItem<T>> + Unpin + $($bounds)* 'static,
            {
                let subject = FluxionSubject::new();
                let subject_clone = subject.clone();

                let task = FluxionTask::spawn(|cancel| async move {
                    let mut stream = source;
                    while let Either::Left((stream_item, _)) =
                        select(stream.next(), cancel.cancelled()).await
                    {
                        match stream_item {
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

            pub fn subscribe(&self) -> Result<SharedBoxStream<T>, SubjectError> {
                Ok(Box::pin(self.subject.subscribe()?))
            }

            pub fn is_closed(&self) -> bool {
                self.subject.is_closed()
            }

            pub fn subscriber_count(&self) -> usize {
                self.subject.subscriber_count()
            }
        }

        impl<T: Clone + $($bounds)* 'static> Drop for FluxionShared<T> {
            fn drop(&mut self) {
                self.subject.close();
            }
        }

        pub trait ShareExt<T: Clone + $($bounds)* 'static>: Stream<Item = StreamItem<T>> {
            fn share(self) -> FluxionShared<T>
            where
                Self: Unpin + $($bounds)* 'static;
        }

        impl<S, T> ShareExt<T> for S
        where
            S: Stream<Item = StreamItem<T>>,
            T: Clone + $($bounds)* 'static,
        {
            fn share(self) -> FluxionShared<T>
            where
                Self: Unpin + $($bounds)* 'static,
            {
                FluxionShared::new(self)
            }
        }
    };
}
