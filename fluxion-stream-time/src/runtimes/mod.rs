// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "time-tokio")]
pub use tokio_impl::tokio_implementation::TokioTimer;

#[cfg(feature = "time-tokio")]
mod tokio_impl;
