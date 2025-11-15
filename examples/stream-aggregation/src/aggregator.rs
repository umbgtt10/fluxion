// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Aggregator module - combines three data sources using FluxionStream

mod event_aggregation;
mod task;

pub use task::Aggregator;
