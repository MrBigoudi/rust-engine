use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn graphics_command_pool_init(&mut self) -> Result<(), EngineError> {
        let pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(self.get_queues()?.graphics_family_index.unwrap() as u32)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        self.context.graphics_command_pool = unsafe {
            let device = self.get_device()?;
            match device.create_command_pool(&pool_create_info, self.get_allocator()?) {
                Ok(pool) => Some(pool),
                Err(err) => {
                    error!(
                        "Failed to create the vulkan graphics command pool: {:?}",
                        err
                    );
                    return Err(EngineError::InitializationFailed);
                }
            }
        };

        Ok(())
    }

    pub fn graphics_command_pool_shutdown(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        unsafe {
            device.destroy_command_pool(*self.get_graphics_command_pool()?, self.get_allocator()?);
        }
        Ok(())
    }

    pub fn get_graphics_command_pool(&self) -> Result<&CommandPool, EngineError> {
        match &self.context.graphics_command_pool {
            Some(pool) => Ok(pool),
            None => {
                error!("Can't access the vulkan graphics command pool");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
