use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{
        vulkan_shaders::builtin_shaders::BuiltinShaders, vulkan_types::VulkanRendererBackend,
    },
};

impl VulkanRendererBackend<'_> {
    pub fn get_builtin_shaders(&self) -> Result<&BuiltinShaders, EngineError> {
        match &self.context.builtin_shaders {
            Some(shaders) => Ok(shaders),
            None => {
                error!("Can't access the vulkan builtin shaders");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn builtin_shaders_init(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        self.context.builtin_shaders = Some(match BuiltinShaders::create(self) {
            Ok(shaders) => shaders,
            Err(err) => {
                error!("Failed to create vulkan builtin shaders: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        });
        Ok(())
    }

    pub fn builtin_shaders_shutdown(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        if let Err(err) = self.get_builtin_shaders()?.destroy(self) {
            error!("Failed to destroy the vulkan builtin shaders: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
        Ok(())
    }
}
