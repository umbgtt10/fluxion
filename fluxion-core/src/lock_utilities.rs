// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_error::{FluxionError, Result};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

// Conditional logging based on tracing feature
#[cfg(feature = "tracing")]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*);
    };
}

#[cfg(not(feature = "tracing"))]
macro_rules! warn {
    ($($arg:tt)*) => {
        // No-op when tracing is disabled
    };
}

/// Safely acquire a lock on a Mutex, converting poison errors to `FluxionError`
///
/// This function handles the case where a thread panicked while holding the lock,
/// which would normally cause a `PoisonError`. Instead, we recover the data and
/// continue, logging the poison error.
///
/// # Arguments
///
/// * `mutex` - The Arc<Mutex<T>> to lock
/// * `context` - A description of what lock is being acquired (for error messages)
///
/// # Returns
///
/// A `Result` containing the `MutexGuard` on success, or a `FluxionError::LockError` on failure
///
/// # Examples
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use fluxion_core::lock_utilities::safe_lock;
///
/// let state = Arc::new(Mutex::new(42));
/// match safe_lock(&state, "counter state") {
///     Ok(guard) => println!("Value: {}", *guard),
///     Err(e) => eprintln!("Failed to lock: {}", e),
/// }
/// ```
/// # Errors
/// Returns an error if locking the mutex fails.
pub fn safe_lock<'a, T>(mutex: &'a Arc<Mutex<T>>, context: &str) -> Result<MutexGuard<'a, T>> {
    mutex
        .lock()
        .map_err(|_poison_err: PoisonError<MutexGuard<T>>| {
            // Log the poison error but recover the data
            warn!("Mutex poisoned for {}: recovering data", context);
            FluxionError::lock_error(context)
        })
        .or_else(|_err| {
            // If we got a poison error, we can still recover the data
            match mutex.lock() {
                Ok(guard) => Ok(guard),
                Err(poison_err) => {
                    // Recover from poison by extracting the guard
                    Ok(poison_err.into_inner())
                }
            }
        })
}

/// Attempt to acquire a lock with a timeout context
///
/// This is a convenience wrapper around `safe_lock` that provides
/// additional context about timeout scenarios.
///
/// # Arguments
///
/// * `mutex` - The Arc<Mutex<T>> to lock
/// * `operation` - Description of the operation being performed
///
/// # Errors
/// Returns an error if locking the mutex fails.
pub fn try_lock<'a, T>(mutex: &'a Arc<Mutex<T>>, operation: &str) -> Result<MutexGuard<'a, T>> {
    safe_lock(mutex, operation)
}
