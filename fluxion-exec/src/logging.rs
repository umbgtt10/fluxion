// Conditional logging shim: uses `tracing` when enabled, falls back to eprintln!/println!

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        tracing::error!($($arg)*);
    }};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        tracing::warn!($($arg)*);
    }};
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        tracing::info!($($arg)*);
    }};
}

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
