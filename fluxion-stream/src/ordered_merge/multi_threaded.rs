// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

define_ordered_merge_impl! {
    inner_bounds: [+ Send + Sync + Unpin],
    timestamp_bounds: [+ Send + Sync + Copy],
    stream_bounds: [Send + Sync +],
    boxed_stream: [Pin<Box<dyn Stream<Item = StreamItem<T>> + Send + Sync>>]
}
