// Conditional logging shim: uses `tracing` when enabled, falls back to eprintln!/println!

#[cfg(feature = "tracing")]
pub use tracing::{error, info, warn};

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
    }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
    }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}

// Note: consider adding lightweight metrics hooks here later
// (e.g., counters for recoverable vs permanent errors) gated by a feature.
