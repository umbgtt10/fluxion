// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Trait for dashboard UI update operations.
///
/// Defines the interface for updating dashboard windows. This abstraction
/// decouples the stream processing logic from the concrete UI implementation,
/// allowing for testing and alternative UI implementations.
pub trait DashboardSink {
    /// Update sensor 1 window with new value
    fn update_sensor1(&mut self, value: u32);

    /// Update sensor 2 window with new value
    fn update_sensor2(&mut self, value: u32);

    /// Update sensor 3 window with new value
    fn update_sensor3(&mut self, value: u32);

    /// Update combined stream window with new value
    fn update_combined(&mut self, value: u32);

    /// Update debounced stream window with new value
    fn update_debounce(&mut self, value: u32);

    /// Update delayed stream window with new value
    fn update_delay(&mut self, value: u32);

    /// Update throttled stream window with new value
    fn update_throttle(&mut self, value: u32);

    /// Update sampled stream window with new value
    fn update_sample(&mut self, value: u32);

    /// Update timeout stream window with new value
    fn update_timeout(&mut self, count: u32);

    /// Display timeout error in timeout window
    fn show_timeout_error(&mut self, error: &str);
}
