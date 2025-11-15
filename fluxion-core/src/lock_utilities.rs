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
/// When a mutex is poisoned, it means a thread panicked while holding the lock,
/// potentially leaving the protected data in an inconsistent state. This function
/// attempts to recover by extracting the guard anyway.
///
/// # Arguments
///
/// * `mutex` - The `Arc<Mutex<T>>` to lock
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
///
/// # Errors
///
/// Returns `FluxionError::LockError` in the following cases:
///
/// - The mutex is poisoned (a thread panicked while holding it) AND recovery fails
/// - Lock acquisition fails for other platform-specific reasons
///
/// Note: In most cases, poisoned locks are recovered automatically by extracting
/// the inner guard. Only unrecoverable lock failures return an error.
///
/// # Recovery Behavior
///
/// If the mutex is poisoned, this function:
/// 1. Logs a warning about the poison error
/// 2. Attempts to extract the guard from the poison error
/// 3. Returns the recovered guard if successful
/// 4. Only returns an error if recovery is impossible
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
/// additional context about timeout scenarios. Currently, it functions
/// identically to `safe_lock` but is provided for semantic clarity when
/// dealing with time-sensitive operations.
///
/// # Arguments
///
/// * `mutex` - The `Arc<Mutex<T>>` to lock
/// * `operation` - Description of the operation being performed
///
/// # Returns
///
/// A `Result` containing the `MutexGuard` on success, or a `FluxionError::LockError` on failure
///
/// # Errors
///
/// Returns `FluxionError::LockError` if the lock cannot be acquired.
/// See [`safe_lock`] for detailed error conditions.
///
/// # Examples
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use fluxion_core::lock_utilities::try_lock;
///
/// let state = Arc::new(Mutex::new(vec![1, 2, 3]));
/// match try_lock(&state, "processing queue") {
///     Ok(mut guard) => guard.push(4),
///     Err(e) => eprintln!("Lock failed: {}", e),
/// }
/// ```
pub fn try_lock<'a, T>(mutex: &'a Arc<Mutex<T>>, operation: &str) -> Result<MutexGuard<'a, T>> {
    safe_lock(mutex, operation)
}
