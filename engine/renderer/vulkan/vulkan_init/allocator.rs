use crate::{
    core::debug::errors::EngineError, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn allocator_init(&mut self) -> Result<(), EngineError> {
        // TODO: build an allocator
        self.context.allocator = None;
        Ok(())
    }

    pub fn allocator_shutdown(&mut self) -> Result<(), EngineError> {
        self.context.allocator = None;
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
