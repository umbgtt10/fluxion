use std::sync::atomic::{AtomicU64, Ordering};

// Single global sequence counter for the entire crate
static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A wrapper that adds automatic timestamping to any value for temporal ordering.
/// 
/// Uses a monotonically increasing sequence counter (logical timestamp) to establish
/// a total ordering of events. The timestamp is assigned when the value is created.
#[derive(Debug, Clone)]
pub struct Timestamped<T> {
    pub value: T,
    sequence: u64,
}

impl<T: PartialEq> PartialEq for Timestamped<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.sequence == other.sequence
    }
}

impl<T: Eq> Eq for Timestamped<T> {}

impl<T> Timestamped<T> {
    /// Creates a new timestamped value with an automatically assigned sequence number.
    pub fn new(value: T) -> Self {
        Self {
            value,
            sequence: GLOBAL_SEQUENCE.fetch_add(1, Ordering::SeqCst),
        }
    }

    /// Gets the inner value, consuming the wrapper.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Gets a reference to the inner value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Gets a mutable reference to the inner value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Gets the sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl<T: PartialEq> PartialOrd for Timestamped<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.sequence.partial_cmp(&other.sequence)
    }
}

impl<T: Eq> Ord for Timestamped<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence.cmp(&other.sequence)
    }
}

impl<T> std::ops::Deref for Timestamped<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Timestamped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Timestamped<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
