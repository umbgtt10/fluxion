// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
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
//!     // Operation that might fail
//!     Err(FluxionError::stream_error("Stream not ready"))
//! }
//! ```

use alloc::format;
use alloc::string::String;
use core::fmt::{self, Display, Formatter};

/// Root error type for all Fluxion operations
///
/// This enum encompasses all possible error conditions that can occur
/// during stream processing, subscription, and channel operations.
#[derive(Debug)]
pub enum FluxionError {
    /// Stream processing encountered an error
    ///
    /// This is a general error for stream operations that don't fit
    /// other specific categories.
    StreamProcessingError {
        /// Description of what went wrong during stream processing
        context: String,
    },

    /// Timeout occurred
    ///
    /// This error is emitted when a time-based operation (like `timeout`)
    /// exceeds its specified duration.
    TimeoutError {
        /// Context about the timeout (e.g. duration)
        context: String,
    },
}

// Manual Display implementation (required for no_std)
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
    /// Create a stream processing error with the given context
    pub fn stream_error(context: impl Into<String>) -> Self {
        Self::StreamProcessingError {
            context: context.into(),
        }
    }

    /// Create a timeout error with the given context
    pub fn timeout_error(context: impl Into<String>) -> Self {
        Self::TimeoutError {
            context: context.into(),
        }
    }

    /// Check if this is a recoverable error
    ///
    /// Some errors indicate transient failures that could succeed on retry.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        false
    }

    /// Check if this error indicates a permanent failure
    ///
    /// Stream processing errors are considered permanent.
    #[must_use]
    pub const fn is_permanent(&self) -> bool {
        matches!(self, Self::StreamProcessingError { .. })
    }
}

/// Specialized Result type for Fluxion operations
///
/// This is a type alias for `std::result::Result<T, FluxionError>`, providing
/// a convenient shorthand for functions that return Fluxion errors.
///
/// # Examples
///
/// ```
/// use fluxion_core::Result;
///
/// fn process() -> Result<String> {
///     Ok("processed".to_string())
/// }
/// ```
pub type Result<T> = core::result::Result<T, FluxionError>;

/// Helper trait for adding context to `Result`s
///
/// This allows chaining context information onto errors in a fluent style.
pub trait ResultExt<T> {
    /// Add context to an error
    ///
    /// # Errors
    /// Returns `Err(FluxionError)` if the underlying result is `Err`.
    fn context(self, context: impl Into<String>) -> Result<T>;

    /// Add context to an error using a closure (lazy evaluation)
    ///
    /// # Errors
    /// Returns `Err(FluxionError)` if the underlying result is `Err`.
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
