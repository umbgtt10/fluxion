use fluxion_core::{HasTimestamp, Timestamped};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestWrapper<T> {
    timestamp: u64,
    value: T,
}

impl<T> TestWrapper<T> {
    pub fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> HasTimestamp for TestWrapper<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T> Timestamped for TestWrapper<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.value
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            value,
            timestamp,
        }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self {
            value,
            timestamp: 999999, // Use a dummy timestamp for tests
        }
    }
}
