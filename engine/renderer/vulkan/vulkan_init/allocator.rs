use crate::{
    core::debug::errors::EngineError, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn init_allocator(&mut self) -> Result<(), EngineError> {
        self.context.allocator = None;
        Ok(())
    }

    pub fn shutdown_allocator(&mut self) -> Result<(), EngineError> {
        Ok(())
    }
}

impl<'a> VulkanRendererBackend<'a> {
    pub fn get_allocator(
        &self,
    ) -> Result<Option<&'a ash::vk::AllocationCallbacks<'a>>, EngineError> {
        Ok(self.context.allocator)
    }
}
