// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub trait DashboardSink {
    fn update_sensor1(&mut self, value: u32);
    fn update_sensor2(&mut self, value: u32);
    fn update_sensor3(&mut self, value: u32);
    fn update_combined(&mut self, value: u32);
    fn update_debounce(&mut self, value: u32);
    fn update_delay(&mut self, value: u32);
    fn update_throttle(&mut self, value: u32);
    fn update_sample(&mut self, value: u32);
    fn update_timeout(&mut self, count: u32);
    fn show_timeout_error(&mut self, error: &str);
}
