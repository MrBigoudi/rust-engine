use crate::{core::debug::errors::EngineError, platforms::platform::Platform};

use super::vulkan_types::VulkanRendererBackend;

pub mod allocator;
pub mod entry;
pub mod instance;

impl VulkanRendererBackend<'_> {
    pub fn init_vulkan(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        self.init_entry()?;
        self.init_allocator()?;
        self.init_instance(application_name, platform)?;
        Ok(())
    }

    pub fn shutdown_vulkan(&mut self) -> Result<(), EngineError> {
        self.shutdown_instance()?;
        self.shutdown_allocator()?;
        self.shutdown_entry()?;
        Ok(())
    }
}
