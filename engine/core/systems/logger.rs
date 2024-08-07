use std::{fs::File, io::Write, path::PathBuf, sync::Mutex};

use once_cell::sync::Lazy;

use crate::{
    core::debug::errors::EngineError,
    platforms::{platform::Platform, platform_linux::PlatformLinux},
};

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
            let msg = format!("[{}] ({}:{})\n", $level, file!(), line!());
            $crate::core::systems::logger::print_console_error()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        } else {
            let msg = format!("[{}] ({}:{})\n", $level, file!(), line!());
            $crate::core::systems::logger::print_console()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $level.is_an_error() {
            let msg = format!("[{}] ({}:{}) {}\n", $level, file!(), line!(), format!($($arg)*));
            $crate::core::systems::logger::print_console_error()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        } else {
            let msg = format!("[{}] ({}:{}) {}\n", $level, file!(), line!(), format!($($arg)*));
            $crate::core::systems::logger::print_console()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        }
    };
}

/// Macro to log without the line number and file information
#[macro_export]
macro_rules! log_no_details {
    ($level:expr) => {
        if $level.is_an_error() {
            let msg = format!("[{}]\n", $level);
            $crate::core::systems::logger::print_console_error()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        } else {
            let msg = format!("[{}]\n", $level);
            $crate::core::systems::logger::print_console()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        }
    };
    ($level:expr, $($arg:tt)*) => {
        if $level.is_an_error() {
            let msg = format!("[{}] {}\n", $level, format!($($arg)*));
            $crate::core::systems::logger::print_console_error()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        } else {
            let msg = format!("[{}] {}\n", $level, format!($($arg)*));
            $crate::core::systems::logger::print_console()(&msg, $level);
            $crate::core::systems::logger::append_to_log_file(&msg);
        }
    };
}

#[macro_export]
macro_rules! error {
    () => {
        $crate::log!($crate::core::systems::logger::LogLevel::Error);
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::systems::logger::LogLevel::Error, $($arg)*);
    }};
}

#[macro_export]
macro_rules! warn {
    () => {
        $crate::log!($crate::core::systems::logger::LogLevel::Warning)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::systems::logger::LogLevel::Warning, $($arg)*)
    }};
}

#[macro_export]
macro_rules! debug {
    () => {
        $crate::log!($crate::core::systems::logger::LogLevel::Debug)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::systems::logger::LogLevel::Debug, $($arg)*)
    }};
}

#[macro_export]
macro_rules! info {
    () => {
        $crate::log!($crate::core::systems::logger::LogLevel::Info)
    };
    ($($arg:tt)*) => {{
        $crate::log!($crate::core::systems::logger::LogLevel::Info, $($arg)*)
    }};
}

#[macro_export]
macro_rules! error_no_details {
    () => {
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Error);
    };
    ($($arg:tt)*) => {{
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Error, $($arg)*);
    }};
}

#[macro_export]
macro_rules! warn_no_details {
    () => {
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Warning)
    };
    ($($arg:tt)*) => {{
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Warning, $($arg)*)
    }};
}

#[macro_export]
macro_rules! debug_no_details {
    () => {
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Debug)
    };
    ($($arg:tt)*) => {{
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Debug, $($arg)*)
    }};
}

#[macro_export]
macro_rules! info_no_details {
    () => {
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Info)
    };
    ($($arg:tt)*) => {{
        $crate::log_no_details!($crate::core::systems::logger::LogLevel::Info, $($arg)*)
    }};
}

#[derive(Default)]
pub(crate) struct Logger {
    pub log_file_path: Option<PathBuf>,
}

pub(crate) static mut GLOBAL_LOGGER: Lazy<Mutex<Logger>> = Lazy::new(Mutex::default);

pub(crate) fn fetch_global_logger(error: EngineError) -> Result<&'static mut Logger, EngineError> {
    unsafe {
        match GLOBAL_LOGGER.get_mut() {
            Ok(logger) => Ok(logger),
            Err(err) => {
                error!("Failed to fetch the global logger: {:?}", err);
                Err(error)
            }
        }
    }
}

pub fn append_to_log_file(msg: &String) {
    let global_logger = match fetch_global_logger(EngineError::InitializationFailed) {
        Ok(logger) => logger,
        Err(_) => panic!("Failed to fetch the global logger!"),
    };
    if let Some(path) = &global_logger.log_file_path {
        // append to log file
        let mut file = match File::options().append(true).open(path) {
            Ok(file) => file,
            Err(err) => {
                panic!(
                    "Failed to open the global logger file {:?}: {:?}",
                    path, err
                );
            }
        };
        if let Err(err) = file.write_all(msg.as_bytes()) {
            panic!(
                "Failed to write to the global logger file {:?}: {:?}",
                path, err
            );
        }
    }
}

/// Initiate the engine logger
pub(crate) fn logger_init() -> Result<(), EngineError> {
    let global_logger = fetch_global_logger(EngineError::InitializationFailed)?;
    let crate_path = env!("CARGO_MANIFEST_DIR");
    let logger_file_name = "console.log";
    // Create a PathBuf to handle the file path
    let logger_file: PathBuf = [crate_path, logger_file_name].iter().collect();
    global_logger.log_file_path = Some(logger_file.clone());

    // clear file
    if let Err(err) = File::create(&logger_file) {
        error!("Failed to initialize the logger: {:?}", err);
        return Err(EngineError::InitializationFailed);
    }
    Ok(())
}

/// Shutdown the engine logger
pub(crate) fn logger_shutdown() -> Result<(), EngineError> {
    unsafe { GLOBAL_LOGGER = Lazy::new(Mutex::default) };
    Ok(())
}
