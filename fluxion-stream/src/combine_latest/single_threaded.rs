// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

crate::define_combine_latest_impl! {
    inner_bounds: [+ Unpin],
    timestamp_bounds: [+ Copy],
    stream_bounds: [],
    state_bounds: [],
    boxed_stream: [Pin<Box<dyn Stream<Item = StreamItem<T>>>>]
}
