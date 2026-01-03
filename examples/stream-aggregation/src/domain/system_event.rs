// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! System event domain type

use fluxion_rx::prelude::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: String,
}

impl HasTimestamp for SystemEvent {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for SystemEvent {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
