// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        {
            use cortex_m_semihosting::hprintln;
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
