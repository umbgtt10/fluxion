// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

macro_rules! define_distinct_until_changed_by_impl {
    ($($bounds:tt)*) => {
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use core::fmt::Debug;
        use fluxion_core::fluxion_mutex::Mutex;
        use fluxion_core::{Fluxion, StreamItem};
        use futures::stream::StreamExt;
        use futures::Stream;

        pub trait DistinctUntilChangedByExt<T>: Stream<Item = StreamItem<T>> + Sized
        where
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn distinct_until_changed_by<F>(
                self,
                compare: F,
            ) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                F: Fn(&T::Inner, &T::Inner) -> bool + $($bounds)* 'static;
        }

        impl<T, S> DistinctUntilChangedByExt<T> for S
        where
            S: Stream<Item = StreamItem<T>> + $($bounds)* 'static,
            T: Fluxion,
            T::Inner: Clone + Debug + Ord + Unpin + $($bounds)* 'static,
            T::Timestamp: Debug + Ord + Copy + $($bounds)* 'static,
        {
            fn distinct_until_changed_by<F>(
                self,
                compare: F,
            ) -> impl Stream<Item = StreamItem<T>> + $($bounds)*
            where
                F: Fn(&T::Inner, &T::Inner) -> bool + $($bounds)* 'static,
            {
                let last_value: Arc<Mutex<Option<T::Inner>>> = Arc::new(Mutex::new(None));
                let compare = Arc::new(compare);

                let stream = self.filter_map(move |item| {
                    let last_value = Arc::clone(&last_value);
                    let compare = Arc::clone(&compare);

                    async move {
                        match item {
                            StreamItem::Value(value) => {
                                let current_inner = value.clone().into_inner();

                                let mut last = last_value.lock();

                                // Check if this value is different from the last emitted value
                                let should_emit = match last.as_ref() {
                                    None => true, // First value, always emit
                                    Some(prev) => !compare(&current_inner, prev),
                                };

                                if should_emit {
                                    // Update last value
                                    *last = Some(current_inner);

                                    // Preserve original timestamp
                                    Some(StreamItem::Value(value))
                                } else {
                                    None // Filter out duplicate
                                }
                            }
                            StreamItem::Error(e) => Some(StreamItem::Error(e)), // Propagate errors
                        }
                    }
                });

                Box::pin(stream)
            }
        }
    };
}
