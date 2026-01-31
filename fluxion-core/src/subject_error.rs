// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::fmt;

/// Errors specific to subject operations (lifecycle and broadcasting).
///
/// These errors represent subject-specific failures distinct from stream processing errors.
/// They can be converted to `FluxionError` when needed for stream propagation.
///
/// # Examples
///
/// ```
/// use fluxion_core::SubjectError;
///
/// let error = SubjectError::Closed;
/// assert_eq!(error.to_string(), "Subject is closed");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectError {
    Closed,
}

impl fmt::Display for SubjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "Subject is closed"),
        }
    }
}

impl core::error::Error for SubjectError {}
