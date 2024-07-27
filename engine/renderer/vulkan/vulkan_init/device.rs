use ash::Device;

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn device_init(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn device_shutdown(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn get_device(&self) -> Result<&Device, EngineError> {
        match &self.context.device {
            Some(device) => Ok(device),
            None => {
                error!("Can't access the vulkan device");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
