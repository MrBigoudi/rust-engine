use crate::{
    core::{
        application::{application_init, Application, ApplicationParameters},
        errors::EngineError,
        systems::{subsystems_init, subsystems_shutdown},
    },
    debug, error,
    game::Game,
    renderer::renderer_frontend::{renderer_init, renderer_shutdown},
};

/// Static variable to allow only a single instantiation of the engine
static mut IS_ENGINE_INITIALIZED: bool = false;

/// Initiatlize the engine
/// Can only be called once
fn engine_init(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<Application, EngineError> {
    // Initialization
    if unsafe { IS_ENGINE_INITIALIZED } {
        error!("The engine is already initialized!");
        return Err(EngineError::MultipleInstantiation);
    }

    let app_name = parameters.application_name.clone();

    match subsystems_init() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the subsystems: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }
    debug!("Subsystems initialized");

    let application = match application_init(parameters, game) {
        Ok(application) => application,
        Err(err) => {
            error!("Failed to create the application: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    };
    debug!("Application initialized");

    match renderer_init(&app_name.clone(), application.platform.as_ref()) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the renderer: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }
    debug!("Renderer initialized");

    unsafe { IS_ENGINE_INITIALIZED = true };

    Ok(application)
}

/// Main loop
fn game_loop(application: &mut Application) -> Result<(), EngineError> {
    match application.run() {
        Ok(()) => Ok(()),
        Err(err) => {
            error!("The application failed to run: {:?}", err);
            Err(EngineError::InitializationFailed)
        }
    }
}

/// Cleanup the engine
fn engine_shutdown(application: &mut Application) -> Result<(), EngineError> {
    match renderer_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the renderer: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    }
    debug!("Renderer shutted down");

    match application.shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the application: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    };
    debug!("Application shutted down");

    match application.game.shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the game: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    };
    debug!("Game shutted down");

    match subsystems_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the subsystems: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    }
    debug!("Subsystems shutted down");

    Ok(())
}

/// Entry point of the game engine
pub fn engine_start(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<(), EngineError> {
    // Initialization
    let mut application = match engine_init(parameters, game) {
        Ok(application) => application,
        Err(err) => {
            error!("Failed to initialize the engine: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    };
    debug!("Engine initialized");

    // Game loop
    game_loop(&mut application)?;

    // Cleanup
    match engine_shutdown(&mut application) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the engine: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    }

    debug!("Engine shutted down");

    Ok(())
}
