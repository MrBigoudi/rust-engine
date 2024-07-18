use crate::{platforms::platform::Platform, renderer::renderer_backend::RendererBackend};

#[derive(Default)]
pub(crate) struct VulkanRendererBackend {
    frame_number: u64,
}

impl RendererBackend for VulkanRendererBackend {
    fn init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), crate::core::errors::EngineError> {
        todo!()
    }

    fn shutdown(&mut self) -> Result<(), crate::core::errors::EngineError> {
        todo!()
    }

    fn resize(&mut self, width: u16, height: u16) -> Result<(), crate::core::errors::EngineError> {
        todo!()
    }

    fn begin_frame(&self, delta_time: f64) -> Result<(), crate::core::errors::EngineError> {
        todo!()
    }

    fn end_frame(&self, delta_time: f64) -> Result<(), crate::core::errors::EngineError> {
        todo!()
    }

    fn increase_frame_number(&mut self) -> Result<(), crate::core::errors::EngineError> {
        self.frame_number += 1;
        Ok(())
    }

    fn get_frame_number(&self) -> Result<u64, crate::core::errors::EngineError> {
        Ok(self.frame_number)
    }
}
