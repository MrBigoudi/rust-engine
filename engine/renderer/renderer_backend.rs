use crate::{core::debug::errors::EngineError, error, platforms::platform::Platform};

use super::{renderer_types::RendererBackendType, vulkan::vulkan_types::VulkanRendererBackend};

pub(crate) trait RendererBackend {
    fn init(&mut self, application_name: &str, platform: &dyn Platform) -> Result<(), EngineError>;

    fn shutdown(&mut self) -> Result<(), EngineError>;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), EngineError>;

    /// Returns true if the frame had begun correctly
    fn begin_frame(&mut self, delta_time: f64) -> Result<bool, EngineError>;

    fn end_frame(&mut self, delta_time: f64) -> Result<(), EngineError>;

    fn increase_frame_number(&mut self) -> Result<(), EngineError>;

    fn get_frame_number(&self) -> Result<u64, EngineError>;
}

pub(crate) fn renderer_backend_init(
    renderer_type: RendererBackendType,
    application_name: &str,
    platform: &dyn Platform,
) -> Result<impl RendererBackend, EngineError> {
    match renderer_type {
        RendererBackendType::Vulkan => {
            let mut backend = VulkanRendererBackend::default();
            match backend.init(application_name, platform) {
                Ok(backend) => backend,
                Err(err) => {
                    error!("Failed to init the Vulkan renderer backend: {:?}", err);
                    return Err(EngineError::InitializationFailed);
                }
            }
            Ok(backend)
        }
        RendererBackendType::OpenGl => {
            error!("The OpenGL backend is not yet implemented");
            Err(EngineError::NotImplemented)
        }
        RendererBackendType::DirectX => {
            error!("The DirectX backend is not yet implemented");
            Err(EngineError::NotImplemented)
        }
    }
}
