// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::StreamItem;
use fluxion_stream_time::SmolTimestamped;
use futures::channel::mpsc;
use futures::stream::{Stream, StreamExt};

pub fn test_channel<T>() -> (
    mpsc::UnboundedSender<SmolTimestamped<T>>,
    impl Stream<Item = StreamItem<SmolTimestamped<T>>>,
) {
    let (tx, rx) = mpsc::unbounded();
    let stream = rx.map(StreamItem::Value);
    (tx, stream)
}

// Simple test data
#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

pub fn person_alice() -> Person {
    Person {
        name: "Alice".to_string(),
        age: 30,
    }
}

#[allow(dead_code)]
pub async fn unwrap_stream<S, T>(
    stream: &mut S,
    _timeout_ms: u64,
) -> Result<T, fluxion_core::FluxionError>
where
    S: Stream<Item = StreamItem<T>> + Unpin,
{
    use futures::StreamExt;

    match stream.next().await {
        Some(StreamItem::Value(v)) => Ok(v),
        Some(StreamItem::Error(e)) => Err(e),
        None => Err(fluxion_core::FluxionError::stream_error("Stream ended")),
    }
}
