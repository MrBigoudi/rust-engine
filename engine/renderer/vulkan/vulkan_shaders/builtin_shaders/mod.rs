pub mod object_shaders;

use object_shaders::ObjectShaders;

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

pub(crate) struct BuiltinShaders {
    pub object_shaders: ObjectShaders,
}

impl BuiltinShaders {
    pub fn create(backend: &VulkanRendererBackend<'_>) -> Result<Self, EngineError> {
        let object_shaders = match ObjectShaders::create(backend) {
            Ok(shader) => shader,
            Err(err) => {
                error!(
                    "Failed to create the object shaders of the builtin vulkan shaders: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        Ok(BuiltinShaders { object_shaders })
    }

    pub fn destroy(&self, backend: &VulkanRendererBackend<'_>) -> Result<(), EngineError> {
        if let Err(err) = self.object_shaders.destroy(backend) {
            error!(
                "Failed to destroy the object shaders of the builtin vulkan shaders: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }
        Ok(())
    }
}
