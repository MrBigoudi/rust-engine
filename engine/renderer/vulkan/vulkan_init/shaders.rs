use crate::{core::debug::errors::EngineError, error, renderer::vulkan::{vulkan_shaders::builtin_shaders::BuiltinShaders, vulkan_types::VulkanRendererBackend}};

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
        self.context.builtin_shaders = Some(BuiltinShaders::create(device)?);
        Ok(())
    }

    pub fn builtin_shaders_shutdown(&mut self) -> Result<(), EngineError>{
        todo!()
    }
}