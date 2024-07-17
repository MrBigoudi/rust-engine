use crate::{debug, error};

pub mod events;
pub mod input;
pub mod logger;

/// Initialize the different subsystems
pub(crate) fn subsystems_init() -> Result<(), super::errors::EngineError> {
    match logger::logger_init() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the logger system: {:?}", err);
            return Err(super::errors::EngineError::InitializationFailed);
        }
    }
    debug!("Logger subsystem initialized");

    match events::events_init() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the events system: {:?}", err);
            return Err(super::errors::EngineError::InitializationFailed);
        }
    }
    debug!("Events subsystem initialized");

    match input::input_init() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the input system: {:?}", err);
            return Err(super::errors::EngineError::InitializationFailed);
        }
    }
    debug!("Input subsystem initialized");

    Ok(())
}

/// Shutdown the different subsystems
pub(crate) fn subsystems_shutdown() -> Result<(), super::errors::EngineError> {
    match input::input_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the input system: {:?}", err);
            return Err(super::errors::EngineError::ShutdownFailed);
        }
    }
    debug!("Input subsystem shutted down");

    match events::events_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the events system: {:?}", err);
            return Err(super::errors::EngineError::ShutdownFailed);
        }
    }
    debug!("Events subsystem shutted down");

    match logger::logger_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the logger system: {:?}", err);
            return Err(super::errors::EngineError::ShutdownFailed);
        }
    }
    debug!("Logger subsystem shutted down");

    Ok(())
}
