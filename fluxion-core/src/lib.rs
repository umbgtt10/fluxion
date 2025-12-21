// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod cancellation_token;
pub mod fluxion;
pub mod fluxion_error;
pub mod fluxion_subject;
pub mod fluxion_task;
pub mod has_timestamp;
pub mod into_stream;
pub mod stream_item;
pub mod subject_error;
pub mod timestamped;

pub use self::cancellation_token::CancellationToken;
pub use self::fluxion::Fluxion;
pub use self::fluxion_error::{FluxionError, IntoFluxionError, Result, ResultExt};
pub use self::fluxion_subject::FluxionSubject;
pub use self::fluxion_task::FluxionTask;
pub use self::has_timestamp::HasTimestamp;
pub use self::into_stream::IntoStream;
pub use self::stream_item::StreamItem;
pub use self::subject_error::SubjectError;
pub use self::timestamped::Timestamped;
