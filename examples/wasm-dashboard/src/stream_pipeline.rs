// Stream pipeline with time operators
use fluxion_core::Stream;
use fluxion_stream_time::{debounce, delay, sample, throttle, timeout};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct StreamPipeline {
    // Operator configurations
    debounce_duration: Arc<Mutex<Duration>>,
    throttle_duration: Arc<Mutex<Duration>>,
    delay_duration: Arc<Mutex<Duration>>,
    sample_interval: Arc<Mutex<Duration>>,
    timeout_duration: Arc<Mutex<Duration>>,

    // Operator enabled flags
    debounce_enabled: Arc<Mutex<bool>>,
    throttle_enabled: Arc<Mutex<bool>>,
    delay_enabled: Arc<Mutex<bool>>,
    sample_enabled: Arc<Mutex<bool>>,
    timeout_enabled: Arc<Mutex<bool>>,
}

impl StreamPipeline {
    pub fn new() -> Self {
        Self {
            debounce_duration: Arc::new(Mutex::new(Duration::from_millis(100))),
            throttle_duration: Arc::new(Mutex::new(Duration::from_millis(100))),
            delay_duration: Arc::new(Mutex::new(Duration::from_millis(50))),
            sample_interval: Arc::new(Mutex::new(Duration::from_millis(200))),
            timeout_duration: Arc::new(Mutex::new(Duration::from_millis(500))),

            debounce_enabled: Arc::new(Mutex::new(false)),
            throttle_enabled: Arc::new(Mutex::new(false)),
            delay_enabled: Arc::new(Mutex::new(false)),
            sample_enabled: Arc::new(Mutex::new(false)),
            timeout_enabled: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_debounce_duration(&self, ms: u32) {
        *self.debounce_duration.lock().unwrap() = Duration::from_millis(ms as u64);
    }

    pub fn set_throttle_duration(&self, ms: u32) {
        *self.throttle_duration.lock().unwrap() = Duration::from_millis(ms as u64);
    }

    pub fn set_delay_duration(&self, ms: u32) {
        *self.delay_duration.lock().unwrap() = Duration::from_millis(ms as u64);
    }

    pub fn set_sample_interval(&self, ms: u32) {
        *self.sample_interval.lock().unwrap() = Duration::from_millis(ms as u64);
    }

    pub fn set_timeout_duration(&self, ms: u32) {
        *self.timeout_duration.lock().unwrap() = Duration::from_millis(ms as u64);
    }

    pub fn toggle_debounce(&self, enabled: bool) {
        *self.debounce_enabled.lock().unwrap() = enabled;
    }

    pub fn toggle_throttle(&self, enabled: bool) {
        *self.throttle_enabled.lock().unwrap() = enabled;
    }

    pub fn toggle_delay(&self, enabled: bool) {
        *self.delay_enabled.lock().unwrap() = enabled;
    }

    pub fn toggle_sample(&self, enabled: bool) {
        *self.sample_enabled.lock().unwrap() = enabled;
    }

    pub fn toggle_timeout(&self, enabled: bool) {
        *self.timeout_enabled.lock().unwrap() = enabled;
    }

    /// Apply the configured operators to a stream
    pub async fn apply<T, S>(&self, stream: S) -> impl Stream<Item = T>
    where
        T: Clone + Send + 'static,
        S: Stream<Item = T> + Send + 'static,
    {
        let mut result: Box<dyn Stream<Item = T>> = Box::new(stream);

        // Apply debounce if enabled
        if *self.debounce_enabled.lock().unwrap() {
            let duration = *self.debounce_duration.lock().unwrap();
            result = Box::new(debounce(result, duration));
        }

        // Apply throttle if enabled
        if *self.throttle_enabled.lock().unwrap() {
            let duration = *self.throttle_duration.lock().unwrap();
            result = Box::new(throttle(result, duration));
        }

        // Apply delay if enabled
        if *self.delay_enabled.lock().unwrap() {
            let duration = *self.delay_duration.lock().unwrap();
            result = Box::new(delay(result, duration));
        }

        // Apply sample if enabled
        if *self.sample_enabled.lock().unwrap() {
            let interval = *self.sample_interval.lock().unwrap();
            result = Box::new(sample(result, interval));
        }

        // Apply timeout if enabled
        if *self.timeout_enabled.lock().unwrap() {
            let duration = *self.timeout_duration.lock().unwrap();
            result = Box::new(timeout(result, duration));
        }

        result
    }
}

impl Default for StreamPipeline {
    fn default() -> Self {
        Self::new()
    }
}
