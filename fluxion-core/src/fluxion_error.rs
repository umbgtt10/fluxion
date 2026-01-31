// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Error types for Fluxion reactive streaming library
//!
//! This module provides a comprehensive error handling system for all Fluxion operations.
//! It defines a root [`FluxionError`] type with specific variants for different failure modes,
//! allowing library users to handle errors appropriately.
//!
//! # Examples
//!
//! ```
//! use fluxion_core::{FluxionError, Result};
//!
//! fn process_data() -> Result<()> {
//!     Err(FluxionError::stream_error("Stream not ready"))
//! }
//!
//! fn process() -> Result<String> {
//!     Ok("processed".to_string())
//! }
//! ```

use alloc::format;
use alloc::string::String;
use core::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum FluxionError {
    StreamProcessingError { context: String },
    TimeoutError { context: String },
}

impl Display for FluxionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::StreamProcessingError { context } => {
                write!(f, "Stream processing error: {}", context)
            }
            Self::TimeoutError { context } => write!(f, "Timeout error: {}", context),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FluxionError {}

impl FluxionError {
    pub fn stream_error(context: impl Into<String>) -> Self {
        Self::StreamProcessingError {
            context: context.into(),
        }
    }

    pub fn timeout_error(context: impl Into<String>) -> Self {
        Self::TimeoutError {
            context: context.into(),
        }
    }

    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        false
    }

    #[must_use]
    pub const fn is_permanent(&self) -> bool {
        matches!(self, Self::StreamProcessingError { .. })
    }
}

pub type Result<T> = core::result::Result<T, FluxionError>;

pub trait ResultExt<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for core::result::Result<T, E>
where
    E: Into<FluxionError>,
{
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let context = context.into();
            let error = e.into();
            match error {
                FluxionError::StreamProcessingError { context: inner } => {
                    FluxionError::StreamProcessingError {
                        context: format!("{context}: {inner}"),
                    }
                }
                other => other,
            }
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let context = f();
            let error = e.into();
            match error {
                FluxionError::StreamProcessingError { context: inner } => {
                    FluxionError::StreamProcessingError {
                        context: format!("{context}: {inner}"),
                    }
                }
                other => other,
            }
        })
    }
}

impl Clone for FluxionError {
    fn clone(&self) -> Self {
        match self {
            Self::StreamProcessingError { context } => Self::StreamProcessingError {
                context: context.clone(),
            },
            Self::TimeoutError { context } => Self::TimeoutError {
                context: context.clone(),
            },
        }
    }
}
