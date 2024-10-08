use ash::Entry;

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn get_entry(&self) -> Result<&Entry, EngineError> {
        match &self.context.entry {
            Some(entry) => Ok(entry),
            None => {
                error!("Can't access the vulkan entry");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn entry_init(&mut self) -> Result<(), EngineError> {
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

    pub fn entry_shutdown(&mut self) -> Result<(), EngineError> {
        self.context.entry = None;
        Ok(())
    }
}
