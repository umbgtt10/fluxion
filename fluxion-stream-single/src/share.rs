// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Share operator - WASM compatible
//!
//! This operator requires dynamic task spawning via FluxionTask and is available for:
//! - WASM (wasm32 target)
//!
//! **Not available in Embassy** - Embassy cannot dynamically spawn tasks.

use fluxion_core::StreamItem;
use futures::Stream;

pub use fluxion_stream_core::{FluxionShared, SharedBoxStream};

/// Extension trait providing the `share` operator for streams.
///
/// Available for WASM. Not available in Embassy.
pub trait ShareExt<T>: Stream<Item = StreamItem<T>> + Sized
where
    T: Clone + Send + Sync + Unpin + 'static,
{
    /// Converts this stream into a shared, multi-subscriber source.
    ///
    /// The source stream is consumed once, and all emitted items are broadcast
    /// to all active subscribers.
    ///
    /// # Behavior
    ///
    /// - **Hot**: Late subscribers do not receive past items
    /// - **Shared execution**: Source runs once; results are broadcast
    /// - **Owned lifecycle**: Forwarding task cancelled when dropped
    fn share(self) -> FluxionShared<T>
    where
        Self: Send + Unpin + 'static,
    {
        FluxionShared::new(self)
    }
}

impl<S, T> ShareExt<T> for S
where
    S: Stream<Item = StreamItem<T>>,
    T: Clone + Send + Sync + Unpin + 'static,
{
}
