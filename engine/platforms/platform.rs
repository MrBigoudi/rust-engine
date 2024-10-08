use crate::{
    core::{debug::errors::EngineError, systems::logger::LogLevel},
    error,
    renderer::vulkan::vulkan_types::VulkanContext,
};

/// Abstract trait for the platform (os) specific code
pub(crate) trait Platform {
    /// Initiate the internal structure of the platform
    fn init(
        &mut self,
        window_title: String,
        x: i16,
        y: i16,
        width: u32,
        height: u32,
        resizable: bool,
    ) -> Result<(), EngineError>;

    /// Shutdown the platform
    fn shutdown(&mut self) -> Result<(), EngineError>;

    /// Loop through all the events until they are all consumed
    /// Return true if should quit
    fn handle_events(&mut self) -> Result<bool, EngineError>;

    /// Ellapsed time in seconds since the UNIX_EPOCH
    /// Panic if an error occurs
    fn get_absolute_time_in_seconds(&self) -> Result<f64, EngineError> {
        error!("Function `get_absolute_time_in_seconds' is not implemented for this platform");
        Err(EngineError::NotImplemented)
    }

    /// Multithreading compatible sleep
    fn sleep_from_milliseconds(&self, ms: u64) -> Result<(), EngineError> {
        error!("Function `sleep_from_milliseconds' is not implemented for this platform");
        Err(EngineError::NotImplemented)
    }

    /// Get the required extensions for the renderer
    fn get_required_extensions(&self) -> Result<Vec<*const i8>, EngineError>;
    /// Defaut output on the console
    fn console_write(message: &str, _log_level: LogLevel)
    where
        Self: Sized,
    {
        print!("{}", message);
    }

    /// Get the vulkan surface
    fn get_vulkan_surface(
        &self,
        vulkan_context: &VulkanContext,
    ) -> Result<ash::vk::SurfaceKHR, EngineError>;

    /// Defaut output on the console for errors
    fn console_write_error(message: &str, _log_level: LogLevel)
    where
        Self: Sized,
    {
        eprint!("{}", message);
    }
}

/// Initiate the engine platform depending on the OS
pub(crate) fn platform_init(
    window_title: String,
    x: i16,
    y: i16,
    width: u32,
    height: u32,
    resizable: bool,
) -> Result<impl Platform, EngineError> {
    #[cfg(target_os = "linux")]
    {
        let mut platform_linux = super::platform_linux::PlatformLinux::default();
        let result = platform_linux.init(window_title, x, y, width, height, resizable);
        match result {
            Err(_) => Err(EngineError::InitializationFailed),
            Ok(_) => Ok(platform_linux),
        }
    }
}
