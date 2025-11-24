// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Adapter that wraps legacy Order events with timestamps

use tokio::sync::mpsc::UnboundedReceiver;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::domain::{events::UnifiedEvent, models::Order, TimestampedEvent};

pub fn wrap_orders(
    rx: UnboundedReceiver<Order>,
) -> impl futures::Stream<Item = TimestampedEvent> + Send + Unpin {
    Box::pin(tokio_stream::StreamExt::map(
        UnboundedReceiverStream::new(rx),
        |order| TimestampedEvent::new(UnifiedEvent::OrderReceived(order)),
    ))
}
