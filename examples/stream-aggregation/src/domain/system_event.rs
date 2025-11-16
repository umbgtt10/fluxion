// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! System event domain type

use fluxion_rx::prelude::Ordered;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: String,
}

impl Ordered for SystemEvent {
    type Inner = Self;

    fn get(&self) -> &Self::Inner {
        self
    }

    fn order(&self) -> u64 {
        self.timestamp
    }

    fn with_order(inner: Self::Inner, order: u64) -> Self {
        Self {
            timestamp: order,
            ..inner
        }
    }
}
