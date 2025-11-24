// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Adapter that wraps legacy User records with timestamps
//! This is Pattern 3: Wrapper Ordering from the Integration Guide

use futures::StreamExt;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::domain::{events::UnifiedEvent, models::User, TimestampedEvent};

/// Wrap legacy user records with timestamps
pub fn wrap_users(
    rx: UnboundedReceiver<User>,
) -> impl futures::Stream<Item = TimestampedEvent> + Send + Unpin {
    Box::pin(StreamExt::map(UnboundedReceiverStream::new(rx), |user| {
        TimestampedEvent::new(UnifiedEvent::UserAdded(user))
    }))
}
