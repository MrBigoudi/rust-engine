use crate::{core::debug::errors::EngineError, platforms::platform::Platform};

use super::vulkan_types::VulkanRendererBackend;

pub mod allocator;
pub mod debug;
pub mod device;
pub mod entry;
pub mod instance;
pub mod surface;

impl VulkanRendererBackend<'_> {
    pub fn vulkan_init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        self.entry_init()?;
        self.allocator_init()?;
        self.instance_init(application_name, platform)?;

        #[cfg(debug_assertions)]
        self.debugger_init()?;

        self.surface_init(platform)?;

        // self.physical_device_init()?;
        // self.device_init()?;

        Ok(())
    }

    pub fn vulkan_shutdown(&mut self) -> Result<(), EngineError> {
        // self.device_shutdown()?;
        // self.physical_device_shutdown()?;

        self.surface_shutdown()?;

        #[cfg(debug_assertions)]
        self.debugger_shutdown()?;

        self.instance_shutdown()?;
        self.allocator_shutdown()?;
        self.entry_shutdown()?;

        Ok(())
    }
}
