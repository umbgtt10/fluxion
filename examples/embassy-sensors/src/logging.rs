#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hprintln;
            // Use format_args! to handle arguments safely
            hprintln!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hprintln;
            hprintln!("WARN: {}", format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hprintln;
            hprintln!("ERROR: {}", format_args!($($arg)*));
        }
    };
}
