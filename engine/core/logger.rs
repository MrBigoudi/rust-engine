use crate::platforms::{platform::Platform, platform_linux::PlatformLinux};

pub enum LogLevel {
    Error,
    Warning,
    Debug,
    Info,
}

impl LogLevel {
    pub fn should_panic(&self) -> bool {
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

#[macro_export]
macro_rules! log {
    ($level:expr) => {
        if $level.should_panic() {
            $crate::core::logger::print_console_error()(&format!("[{}]\n", $level), $level)
        } else {
            $crate::core::logger::print_console()(&format!("[{}]\n", $level), $level)
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $level.should_panic() {
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
        panic!()
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::logger::LogLevel::Error, $($arg)*);
        panic!()
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

pub fn init_logger() -> bool {
    // TODO: implement log file
    true
}

pub fn shutdown_logger() {
    // TODO:
}
