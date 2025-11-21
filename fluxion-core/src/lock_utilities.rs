// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::sync::{Arc, Mutex, MutexGuard};

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

/// Safely acquire a lock on a Mutex, recovering from poison errors
///
/// This function handles the case where a thread panicked while holding the lock,
/// which would normally cause a `PoisonError`. Instead of propagating the error,
/// this function recovers the mutex guard and continues execution, logging a warning
/// about the poison.
///
/// When a mutex is poisoned, it means a thread panicked while holding the lock,
/// potentially leaving the protected data in an inconsistent state. This function
/// follows Rust's standard library behavior of allowing recovery from poison by
/// extracting the guard from the PoisonError.
///
/// # Arguments
///
/// * `mutex` - The `Arc<Mutex<T>>` to lock
/// * `context` - A description of what lock is being acquired (for logging)
///
/// # Returns
///
/// Always returns the `MutexGuard`, recovering from poison if necessary.
///
/// # Examples
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use fluxion_core::lock_utilities::lock_or_recover;
///
/// let state = Arc::new(Mutex::new(42));
/// let guard = lock_or_recover(&state, "counter state");
/// println!("Value: {}", *guard);
/// ```
///
/// # Panics
///
/// This function will panic if the underlying mutex implementation fails in a way
/// that doesn't produce a PoisonError (extremely rare, platform-specific failure).
pub fn lock_or_recover<'a, T>(mutex: &'a Arc<Mutex<T>>, _context: &str) -> MutexGuard<'a, T> {
    mutex.lock().unwrap_or_else(|poison_err| {
        warn!("Mutex poisoned for {}: recovering", _context);
        poison_err.into_inner()
    })
}
