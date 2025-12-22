// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![cfg_attr(
    not(any(
        feature = "std",
        feature = "runtime-tokio",
        feature = "runtime-smol",
        feature = "runtime-async-std",
        target_arch = "wasm32"
    )),
    no_std
)]
#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]

extern crate alloc;

pub mod cancellation_token;
pub mod fluxion;
pub mod fluxion_error;
#[cfg(feature = "std")]
pub mod fluxion_subject;
pub mod fluxion_task;
pub mod has_timestamp;
pub mod into_stream;
pub mod stream_item;
#[cfg(feature = "std")]
pub mod subject_error;
pub mod timestamped;

pub use self::cancellation_token::CancellationToken;
pub use self::fluxion::Fluxion;
#[cfg(feature = "std")]
pub use self::fluxion_error::IntoFluxionError;
pub use self::fluxion_error::{FluxionError, Result, ResultExt};
#[cfg(feature = "std")]
pub use self::fluxion_subject::FluxionSubject;
pub use self::fluxion_task::FluxionTask;
pub use self::has_timestamp::HasTimestamp;
pub use self::into_stream::IntoStream;
pub use self::stream_item::StreamItem;
#[cfg(feature = "std")]
pub use self::subject_error::SubjectError;
pub use self::timestamped::Timestamped;
