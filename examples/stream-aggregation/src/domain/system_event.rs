// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! System event domain type

use fluxion_rx::prelude::Timestamped;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: String,
}

impl Timestamped for SystemEvent {
    type Inner = Self;

    fn inner(&self) -> &Self::Inner {
        self
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn with_timestamp(inner: Self::Inner, timestamp: u64) -> Self {
        Self {
            timestamp,
            ..inner
        }
    }

    fn with_fresh_timestamp(value: Self) -> Self {
        value
    }
}
