// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
#[macro_use]
mod logging;
pub mod subscribe_async;
pub mod subscribe_latest_async;

// Re-export commonly used types
pub use subscribe_async::SubscribeAsyncExt;
pub use subscribe_latest_async::SubscribeLatestAsyncExt;
