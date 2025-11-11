pub use fluxion_stream::FluxionChannel;

/// Helper to create multiple test channels at once.
pub struct TestChannels;

impl TestChannels {
    /// Creates three test channels.
    pub fn three<T>() -> (FluxionChannel<T>, FluxionChannel<T>, FluxionChannel<T>) {
        (
            FluxionChannel::new(),
            FluxionChannel::new(),
            FluxionChannel::new(),
        )
    }

    /// Creates two test channels.
    pub fn two<T>() -> (FluxionChannel<T>, FluxionChannel<T>) {
        (FluxionChannel::new(), FluxionChannel::new())
    }
}
