// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

/// Macro that generates the complete shared stream implementation.
///
/// This macro eliminates duplication between multi-threaded and single-threaded
/// implementations, which differ only in trait bounds (Send + Sync vs not).
///
/// The macro generates:
/// - SharedBoxStream type alias
/// - FluxionShared struct
/// - impl with new(), subscribe(), is_closed(), subscriber_count()
/// - Drop implementation
/// - ShareExt trait with share() method
/// - Blanket trait implementation
macro_rules! define_share_impl {
    (
        stream_bounds: [$($stream_bounds:tt)*],
        type_bounds: [$($type_bounds:tt)*],
        share_bounds: [$($share_bounds:tt)*]
    ) => {
        use alloc::boxed::Box;
        use core::pin::Pin;
        use fluxion_core::{FluxionSubject, FluxionTask, StreamItem, SubjectError};
        use futures::{
            future::{select, Either},
            Stream, StreamExt,
        };

        pub type SharedBoxStream<T> = Pin<Box<dyn Stream<Item = StreamItem<T>> $($stream_bounds)* + 'static>>;

        /// A shared stream that broadcasts items from a source to multiple subscribers.
        ///
        /// See the [module-level documentation](crate::share) for detailed examples and usage patterns.
        pub struct FluxionShared<T: Clone $($type_bounds)* + 'static> {
            subject: FluxionSubject<T>,
            _task: FluxionTask,
        }

        impl<T: Clone $($type_bounds)* + 'static> FluxionShared<T> {
            /// Creates a new `FluxionShared` from a source stream.
            ///
            /// Prefer using [`ShareExt::share()`] instead of calling this directly.
            pub fn new<S>(source: S) -> Self
            where
                S: Stream<Item = StreamItem<T>> $($stream_bounds)* + Unpin + 'static,
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

            /// Subscribes to this shared stream, creating a new independent stream.
            pub fn subscribe(&self) -> Result<SharedBoxStream<T>, SubjectError> {
                Ok(Box::pin(self.subject.subscribe()?))
            }

            /// Returns true if the source stream has completed and no more items will be emitted.
            pub fn is_closed(&self) -> bool {
                self.subject.is_closed()
            }

            /// Returns the current number of active subscribers.
            pub fn subscriber_count(&self) -> usize {
                self.subject.subscriber_count()
            }
        }

        impl<T: Clone $($type_bounds)* + 'static> Drop for FluxionShared<T> {
            fn drop(&mut self) {
                self.subject.close();
            }
        }

        /// Extension trait for sharing a stream among multiple subscribers.
        pub trait ShareExt<T: Clone $($type_bounds)* + 'static>: Stream<Item = StreamItem<T>> {
            /// Shares this stream among multiple subscribers.
            ///
            /// See the [module-level documentation](crate::share) for usage examples.
            fn share(self) -> FluxionShared<T>
            where
                Self: $($share_bounds)* + 'static;
        }

        impl<S, T> ShareExt<T> for S
        where
            S: Stream<Item = StreamItem<T>>,
            T: Clone $($type_bounds)* + 'static,
        {
            fn share(self) -> FluxionShared<T>
            where
                Self: $($share_bounds)* + 'static,
            {
                FluxionShared::new(self)
            }
        }
    };
}
