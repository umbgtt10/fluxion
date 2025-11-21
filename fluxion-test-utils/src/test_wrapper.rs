use fluxion_core::Timestamped;

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

impl<T> Timestamped for TestWrapper<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Inner = Self;
    type Timestamp = u64;

    fn into_inner(self) -> Self::Inner {
        self
    }

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self {
            value: value.value,
            timestamp,
        }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self {
            value: value.value,
            timestamp: 999999, // Use a dummy timestamp for tests
        }
    }
}
