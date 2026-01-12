// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use async_channel::Sender;
use fluxion_core::StreamItem;
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;

pub fn test_channel<T>() -> (Sender<T>, Pin<Box<dyn Stream<Item = StreamItem<T>> + Send>>)
where
    T: Send + 'static,
{
    let (tx, rx) = async_channel::unbounded();
    let stream = futures::stream::unfold(rx, |rx| async move {
        rx.recv().await.ok().map(|v| (StreamItem::Value(v), rx))
    });
    (tx, Box::pin(stream))
}

// Simple test data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
