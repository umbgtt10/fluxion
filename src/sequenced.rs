use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A wrapper that adds automatic sequencing to any value for temporal ordering.
/// The sequence number is assigned when the value is created.
#[derive(Debug, Clone)]
pub struct Sequenced<T> {
    pub value: T,
    sequence: u64,
}

impl<T: PartialEq> PartialEq for Sequenced<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.sequence == other.sequence
    }
}

impl<T: Eq> Eq for Sequenced<T> {}

impl<T> Sequenced<T> {
    /// Creates a new sequenced value with an automatically assigned sequence number.
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

impl<T: PartialEq> PartialOrd for Sequenced<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.sequence.partial_cmp(&other.sequence)
    }
}

impl<T: Eq> Ord for Sequenced<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence.cmp(&other.sequence)
    }
}

impl<T> std::ops::Deref for Sequenced<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Sequenced<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Sequenced<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequenced_ordering() {
        let first = Sequenced::new("first");
        let second = Sequenced::new("second");
        let third = Sequenced::new("third");

        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
    }

    #[test]
    fn test_sequenced_deref() {
        let seq = Sequenced::new("hello");
        assert_eq!(seq.len(), 5); // Derefs to &str
    }
}
