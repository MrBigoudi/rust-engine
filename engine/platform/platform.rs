use crate::core::logger::LogLevel;

pub trait Platform {
    fn init(&mut self, window_title: String, x: i16, y: i16, width: u16, height: u16);

    fn shutdown(&mut self);

    /// Return true if should quit
    fn handle_events(&mut self) -> bool;

    fn get_absolute_time_in_seconds(&mut self) -> f64;

    fn sleep_from_milliseconds(&mut self, ms: u64);

    fn console_write(message: &str, _log_level: LogLevel){
        print!("{}", message);
    }

    fn console_write_error(message: &str, _log_level: LogLevel){
        eprint!("{}", message);
    }
}

pub fn init_platform(window_title: String, x: i16, y: i16, width: u16, height: u16) -> Option<impl Platform> {
    #[cfg(target_os = "linux")]
    {
        let mut platform_linux = super::platform_linux::PlatformLinux::default();
        platform_linux.init(window_title, x, y, width, height);
        return Some(platform_linux);
    }
}