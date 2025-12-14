// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

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

/// Root error type for all Fluxion operations
///
/// This enum encompasses all possible error conditions that can occur
/// during stream processing, subscription, and channel operations.
#[derive(Debug, thiserror::Error)]
pub enum FluxionError {
    /// Stream processing encountered an error
    ///
    /// This is a general error for stream operations that don't fit
    /// other specific categories.
    #[error("Stream processing error: {context}")]
    StreamProcessingError {
        /// Description of what went wrong during stream processing
        context: String,
    },

    /// Custom error from user code
    ///
    /// This wraps errors produced by user-provided functions and callbacks,
    /// allowing them to be propagated through the Fluxion error system.
    #[error("User error: {0}")]
    UserError(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// Multiple errors occurred
    ///
    /// When processing multiple items in parallel, multiple failures can occur.
    /// This variant aggregates them.
    #[error("Multiple errors occurred: {count} errors")]
    MultipleErrors {
        /// Number of errors that occurred
        count: usize,
        /// The individual errors (limited to prevent unbounded growth)
        errors: Vec<FluxionError>,
    },

    /// Timeout occurred
    ///
    /// This error is emitted when a time-based operation (like `timeout`)
    /// exceeds its specified duration.
    #[error("Timeout error: {context}")]
    TimeoutError {
        /// Context about the timeout (e.g. duration)
        context: String,
    },
}

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

    /// Wrap a user error
    pub fn user_error(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::UserError(Box::new(error))
    }

    /// Aggregate multiple user errors into a `MultipleErrors` variant
    ///
    /// This is useful for collecting errors from stream subscribers that don't have
    /// error callbacks, allowing them to be propagated as a single error.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluxion_core::FluxionError;
    ///
    /// #[derive(Debug, thiserror::Error)]
    /// #[error("Custom error: {msg}")]
    /// struct CustomError {
    ///     msg: String,
    /// }
    ///
    /// let errors = vec![
    ///     CustomError { msg: "first".to_string() },
    ///     CustomError { msg: "second".to_string() },
    /// ];
    ///
    /// let result = FluxionError::from_user_errors(errors);
    /// assert!(matches!(result, FluxionError::MultipleErrors { count: 2, .. }));
    /// ```
    pub fn from_user_errors<E>(errors: Vec<E>) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let count = errors.len();
        let fluxion_errors = errors
            .into_iter()
            .map(|e| Self::UserError(Box::new(e)))
            .collect();

        Self::MultipleErrors {
            count,
            errors: fluxion_errors,
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
    /// User errors and stream processing errors are considered permanent.
    #[must_use]
    pub const fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::StreamProcessingError { .. } | Self::UserError(_)
        )
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
pub type Result<T> = std::result::Result<T, FluxionError>;

/// Extension trait for converting errors into `FluxionError`
///
/// This trait is automatically implemented for all types that implement
/// `std::error::Error + Send + Sync + 'static`, allowing easy conversion
/// to `FluxionError`.
pub trait IntoFluxionError {
    /// Convert this error into a `FluxionError` with additional context
    fn into_fluxion_error(self, context: &str) -> FluxionError;

    /// Convert this error into a `FluxionError` without additional context
    fn into_fluxion(self) -> FluxionError
    where
        Self: Sized,
    {
        self.into_fluxion_error("")
    }
}

impl<E: std::error::Error + Send + Sync + 'static> IntoFluxionError for E {
    fn into_fluxion_error(self, _context: &str) -> FluxionError {
        FluxionError::user_error(self)
    }
}

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

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<FluxionError>,
{
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let context = context.into();
            match e.into() {
                FluxionError::UserError(inner) => FluxionError::StreamProcessingError {
                    context: format!("{context}: {inner}"),
                },
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
            match e.into() {
                FluxionError::UserError(inner) => FluxionError::StreamProcessingError {
                    context: format!("{context}: {inner}"),
                },
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
            // For UserError, we can't clone the boxed error, so convert to string
            Self::UserError(e) => Self::StreamProcessingError {
                context: format!("User error: {}", e),
            },
            Self::MultipleErrors { count, errors } => Self::MultipleErrors {
                count: *count,
                errors: errors.clone(),
            },
            Self::TimeoutError { context } => Self::TimeoutError {
                context: context.clone(),
            },
        }
    }
}
