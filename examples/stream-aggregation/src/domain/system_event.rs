// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! System event domain type

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: String,
}
