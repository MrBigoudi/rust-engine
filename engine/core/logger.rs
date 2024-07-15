use crate::platforms::{platform::Platform, platform_linux::PlatformLinux};

use super::errors::EngineError;

/// The log levels for the application
pub enum LogLevel {
    /// Fatal errors resulting in a panic
    Error,
    /// Warning not fatals
    Warning,
    /// Debug informations
    Debug,
    /// Other printable information
    Info,
}

impl LogLevel {
    /// Return true if this level of logging is considered as an error
    /// true for Error, false for everything else
    pub fn is_an_error(&self) -> bool {
        match self {
            LogLevel::Error => true,
            LogLevel::Warning => false,
            LogLevel::Debug => false,
            LogLevel::Info => false,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warning => write!(f, "WARNING"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
        }
    }
}

/// Platform specific printer
pub fn print_console() -> fn(&str, LogLevel) {
    #[cfg(target_os = "linux")]
    {
        PlatformLinux::console_write
    }

    #[cfg(not(any(target_os = "linux")))]
    {
        Platform::console_write
    }
}

/// Platform specific printer for errors
pub fn print_console_error() -> fn(&str, LogLevel) {
    #[cfg(target_os = "linux")]
    {
        PlatformLinux::console_write_error
    }

    #[cfg(not(any(target_os = "linux")))]
    {
        Platform::console_write_error
    }
}

/// Macro for for logging message
/// This maccro should not be used on its own but through other macros like error!, warn!, debug! and info!
#[macro_export]
macro_rules! log {
    ($level:expr) => {
        if $level.is_an_error() {
            $crate::core::logger::print_console_error()(&format!("[{}]\n", $level), $level)
        } else {
            $crate::core::logger::print_console()(&format!("[{}]\n", $level), $level)
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $level.is_an_error() {
            $crate::core::logger::print_console_error()(&format!("[{}] {}\n", $level, format!($($arg)*)), $level)
        } else {
            $crate::core::logger::print_console()(&format!("[{}] {}\n", $level, format!($($arg)*)), $level);
        }
    };
}

#[macro_export]
macro_rules! error {
    () => {
        $crate::log!($crate::core::logger::LogLevel::Error);
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::logger::LogLevel::Error, $($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    () => {
        $crate::log!($crate::core::logger::LogLevel::Warning)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::logger::LogLevel::Warning, $($arg)*)
    }};
}

#[macro_export]
macro_rules! debug {
    () => {
        $crate::log!($crate::core::logger::LogLevel::Debug)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::logger::LogLevel::Debug, $($arg)*)
    }};
}

#[macro_export]
macro_rules! info {
    () => {
        $crate::log!($crate::core::logger::LogLevel::Info)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::logger::LogLevel::Info, $($arg)*)
    }};
}

/// Initiate the engine logger
pub fn init_logger() -> Result<(), EngineError> {
    // TODO: implement log file
    Ok(())
}

/// Shutdown the engine logger
pub fn shutdown_logger() -> Result<(), EngineError> {
    // TODO:
    Ok(())
}
