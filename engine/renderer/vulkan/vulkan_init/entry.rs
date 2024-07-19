use ash::Entry;

use crate::{
    core::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn init_entry(&mut self) -> Result<(), EngineError> {
        unsafe {
            self.context.entry = Some(match Entry::load() {
                Ok(entry) => entry,
                Err(err) => {
                    error!("Failed to load the vulkan library: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            });
        }

        Ok(())
    }
}
