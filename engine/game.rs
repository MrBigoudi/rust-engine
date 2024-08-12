use crate::core::debug::errors::EngineError;

/// Game state
/// Called by the application
pub trait Game {
    /// Initializer
    fn on_start(&mut self) -> Result<(), EngineError> {
        Ok(())
    }

    /// Update
    fn on_update(&mut self, delta_time: f64) -> Result<(), EngineError> {
        Ok(())
    }

    /// Render
    fn on_render(&self, delta_time: f64) -> Result<(), EngineError> {
        Ok(())
    }

    /// Resize
    fn on_resize(&mut self, new_width: u32, new_height: u32) -> Result<(), EngineError> {
        Ok(())
    }

    /// Shutdown
    fn on_shutdown(&mut self) -> Result<(), EngineError> {
        Ok(())
    }
}
