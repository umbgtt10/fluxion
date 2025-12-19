#[cfg(feature = "time-tokio")]
pub use tokio_impl::implementation::TokioTimer;

#[cfg(feature = "time-tokio")]
mod tokio_impl;
