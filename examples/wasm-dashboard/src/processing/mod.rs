// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Processing layer - Stream orchestration and business logic.
//!
//! This module provides stream processing, coordination, and wiring logic
//! for the dashboard. It is independent of UI implementation details.

mod combined_stream;
mod dashboard_orchestrator;
mod processing_layer;
mod result_streams;
mod stream_provider;

pub use combined_stream::CombinedStream;
pub use dashboard_orchestrator::DashboardOrchestrator;
pub use processing_layer::ProcessingLayer;
pub use result_streams::{ResultStreams, WasmStream};
pub use stream_provider::StreamProvider;
