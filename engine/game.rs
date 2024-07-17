use crate::core::errors::EngineError;

/// Game state
/// Called by the application
pub trait Game {
    /// Initializer
    fn initialize(&mut self) -> Result<(), EngineError>;

    /// Update
    fn update(&mut self, delta_time: f32) -> Result<(), EngineError>;

    /// Render
    fn render(&self, delta_time: f32) -> Result<(), EngineError>;

    /// Resize
    fn resize(&mut self, new_width: u16, new_height: u16) -> Result<(), EngineError>;

    /// Shutdown
    fn shutdown(&mut self) -> Result<(), EngineError>;
}