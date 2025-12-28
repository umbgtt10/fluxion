use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::EmbassyInstant;

/// Temperature reading in degrees Celsius
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Temperature {
    pub value_kelvin: u32,
    pub timestamp: EmbassyInstant,
}

impl HasTimestamp for Temperature {
    type Timestamp = EmbassyInstant;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for Temperature {
    type Inner = Self;

    fn with_timestamp(inner: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..inner }
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}

impl core::fmt::Display for Temperature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} K", self.value_kelvin)
    }
}
