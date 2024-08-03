use ash::vk::{Semaphore, SemaphoreCreateInfo};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn create_default_semaphore(&self) -> Result<Semaphore, EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        let semaphore_info = SemaphoreCreateInfo::default();
        unsafe {
            match device.create_semaphore(&semaphore_info, allocator) {
                Ok(semaphore) => Ok(semaphore),
                Err(err) => {
                    error!("Failed to create a vulkan semaphore: {:?}", err);
                    Err(EngineError::VulkanFailed)
                }
            }
        }
    }

    pub fn destroy_semaphore(&self, semaphore: &Semaphore) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            device.destroy_semaphore(*semaphore, allocator);
        }
        Ok(())
    }
}
