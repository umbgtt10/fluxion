// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Aggregator module - combines three data sources using FluxionStream

mod aggregation_task;
mod event_aggregation;

pub use aggregation_task::Aggregator;
