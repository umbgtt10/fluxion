// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Helper functions for Embassy tests (similar to WASM test helpers)

use fluxion_core::StreamItem;
use futures::channel::mpsc;
use futures::stream::{Stream, StreamExt};

pub fn test_channel<T>() -> (mpsc::UnboundedSender<T>, impl Stream<Item = StreamItem<T>>) {
    let (tx, rx) = mpsc::unbounded();
    let stream = rx.map(StreamItem::Value);
    (tx, stream)
}

/// Unwrap a single item from the stream with a timeout
pub async fn unwrap_stream<T>(
    stream: &mut (impl Stream<Item = StreamItem<T>> + Unpin),
    timeout_ms: u64,
) -> Option<T> {
    // Use embassy_time for timeout
    let timeout = embassy_time::Duration::from_millis(timeout_ms);

    match embassy_time::with_timeout(timeout, stream.next()).await {
        Ok(Some(StreamItem::Value(v))) => Some(v),
        Ok(Some(StreamItem::Error(_))) => None,
        Ok(None) => None,
        Err(_) => None, // Timeout
    }
}

// Test data structures
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

pub fn person_alice() -> Person {
    Person {
        name: "Alice".to_string(),
        age: 30,
    }
}

pub fn person_bob() -> Person {
    Person {
        name: "Bob".to_string(),
        age: 35,
    }
}
