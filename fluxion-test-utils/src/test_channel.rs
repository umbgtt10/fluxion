pub use fluxion_stream::test_channel::TestChannel;

/// Helper to create multiple test channels at once.
pub struct TestChannels;

impl TestChannels {
    /// Creates three test channels.
    pub fn three<T>() -> (TestChannel<T>, TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new(), TestChannel::new())
    }

    /// Creates two test channels.
    pub fn two<T>() -> (TestChannel<T>, TestChannel<T>) {
        (TestChannel::new(), TestChannel::new())
    }
}
