use ash::{vk::PhysicalDevice, Device};

use crate::{core::debug::errors::EngineError, renderer::vulkan::vulkan_types::VulkanRendererBackend};

impl VulkanRendererBackend<'_>{
    pub fn physical_device_init(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn physical_device_shutdown(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn get_physical_device(&self) -> Result<&PhysicalDevice, EngineError> {
        todo!()
    }


    pub fn device_init(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn device_shutdown(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn get_device(&self) -> Result<&Device, EngineError> {
        todo!()
    }
}