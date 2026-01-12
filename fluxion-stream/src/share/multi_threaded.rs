// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

crate::define_share_impl! {
    stream_bounds: [+ Send + Sync],
    type_bounds: [+ Send + Sync],
    share_bounds: [Send + Sync + Unpin]
}
