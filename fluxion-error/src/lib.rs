#![allow(clippy::multiple_crate_versions)]
//! Error types for Fluxion reactive streaming library
//!
//! This crate provides a comprehensive error handling system for all Fluxion operations.
//! It defines a root [`FluxionError`] type with specific variants for different failure modes,
//! allowing library users to handle errors appropriately.
//!
//! # Examples
//!
//! ```
//! use fluxion_error::{FluxionError, Result};
//!
//! fn process_data() -> Result<()> {
//!     // Operation that might fail
//!     Err(FluxionError::InvalidState {
//!         message: "Stream not ready".to_string(),
//!     })
//! }
//! ```

/// Root error type for all Fluxion operations
///
/// This enum encompasses all possible error conditions that can occur
/// during stream processing, subscription, and channel operations.
#[derive(Debug, thiserror::Error)]
pub enum FluxionError {
    /// Error acquiring a lock on shared state
    ///
    /// This typically indicates contention or a poisoned mutex.
    /// The context provides details about which lock failed.
    #[error("Failed to acquire lock: {context}")]
    LockError {
        /// Description of the lock that failed
        context: String,
    },

    /// Channel send operation failed
    ///
    /// This occurs when attempting to send to a channel whose receiver
    /// has been dropped.
    #[error("Channel send failed: receiver dropped")]
    ChannelSendError,

    /// Channel receive operation failed
    ///
    /// This can occur when the channel is closed, empty, or in an invalid state.
    #[error("Channel receive failed: {reason}")]
    ChannelReceiveError {
        /// Specific reason for the receive failure
        reason: String,
    },

    /// Stream processing encountered an error
    ///
    /// This is a general error for stream operations that don't fit
    /// other specific categories.
    #[error("Stream processing error: {context}")]
    StreamProcessingError {
        /// Description of what went wrong during stream processing
        context: String,
    },

    /// User-provided callback function panicked
    ///
    /// When a user-supplied closure or function panics during stream processing,
    /// it's caught and converted to this error variant.
    #[error("User callback panicked: {context}")]
    CallbackPanic {
        /// Information about the panic location and cause
        context: String,
    },

    /// Subscription operation failed
    ///
    /// This encompasses errors during `subscribe_async` or `subscribe_latest_async`
    /// operations, including user callback errors when no error handler is provided.
    #[error("Subscription error: {context}")]
    SubscriptionError {
        /// Details about the subscription failure
        context: String,
    },

    /// Invalid state encountered
    ///
    /// This indicates that an operation was attempted when the stream or channel
    /// was in an inappropriate state.
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of the invalid state
        message: String,
    },

    /// Timeout occurred while waiting for an operation
    ///
    /// Used when operations have time limits and they expire.
    #[error("Operation timed out after {duration:?}: {operation}")]
    Timeout {
        /// The operation that timed out
        operation: String,
        /// How long we waited
        duration: std::time::Duration,
    },

    /// Stream unexpectedly ended
    ///
    /// This occurs when more items were expected but the stream terminated.
    #[error("Stream ended unexpectedly: expected {expected}, got {actual}")]
    UnexpectedStreamEnd {
        /// Number of items expected
        expected: usize,
        /// Number of items actually received
        actual: usize,
    },

    /// Resource limit exceeded
    ///
    /// This indicates that a buffer, queue, or other bounded resource is full.
    #[error("Resource limit exceeded: {resource} (limit: {limit})")]
    ResourceLimitExceeded {
        /// Name of the resource that hit its limit
        resource: String,
        /// The limit that was exceeded
        limit: usize,
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
}

impl FluxionError {
    /// Create a lock error with the given context
    pub fn lock_error(context: impl Into<String>) -> Self {
        Self::LockError {
            context: context.into(),
        }
    }

    /// Create a stream processing error with the given context
    pub fn stream_error(context: impl Into<String>) -> Self {
        Self::StreamProcessingError {
            context: context.into(),
        }
    }

    /// Create an invalid state error with the given message
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::InvalidState {
            message: message.into(),
        }
    }

    /// Create a subscription error with the given context
    pub fn subscription_error(context: impl Into<String>) -> Self {
        Self::SubscriptionError {
            context: context.into(),
        }
    }

    /// Create a channel receive error with the given reason
    pub fn channel_receive_error(reason: impl Into<String>) -> Self {
        Self::ChannelReceiveError {
            reason: reason.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>, duration: std::time::Duration) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration,
        }
    }

    /// Create an unexpected stream end error
    #[must_use]
    pub const fn unexpected_end(expected: usize, actual: usize) -> Self {
        Self::UnexpectedStreamEnd { expected, actual }
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit(resource: impl Into<String>, limit: usize) -> Self {
        Self::ResourceLimitExceeded {
            resource: resource.into(),
            limit,
        }
    }

    /// Wrap a user error
    pub fn user_error(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::UserError(Box::new(error))
    }

    /// Check if this is a recoverable error
    ///
    /// Some errors indicate transient failures that could succeed on retry.
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::LockError { .. } | Self::Timeout { .. } | Self::ResourceLimitExceeded { .. }
        )
    }

    /// Check if this error indicates a permanent failure
    #[must_use]
    pub const fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::ChannelSendError | Self::ChannelReceiveError { .. } | Self::InvalidState { .. }
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
/// use fluxion_error::Result;
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
