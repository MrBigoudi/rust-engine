use crate::{
    core::debug::errors::EngineError, platforms::platform::Platform,
    renderer::renderer_backend::RendererBackend,
};

use super::vulkan_types::VulkanRendererBackend;

impl RendererBackend for VulkanRendererBackend<'_> {
    fn init(&mut self, application_name: &str, platform: &dyn Platform) -> Result<(), EngineError> {
        self.init_vulkan(application_name, platform)?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), EngineError> {
        self.shutdown_vulkan()?;
        Ok(())
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<(), EngineError> {
        todo!()
    }

    fn begin_frame(&self, delta_time: f64) -> Result<(), EngineError> {
        todo!()
    }

    fn end_frame(&self, delta_time: f64) -> Result<(), EngineError> {
        todo!()
    }

    fn increase_frame_number(&mut self) -> Result<(), EngineError> {
        self.frame_number += 1;
        Ok(())
    }

    fn get_frame_number(&self) -> Result<u64, EngineError> {
        Ok(self.frame_number)
    }
}
