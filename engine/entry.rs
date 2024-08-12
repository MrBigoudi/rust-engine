use crate::{
    core::{
        application::{
            application_init, application_shutdown, fetch_global_application, ApplicationParameters,
        },
        debug::errors::EngineError,
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
fn engine_init(parameters: ApplicationParameters, game: Box<dyn Game>) -> Result<(), EngineError> {
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

    if let Err(err) = application_init(parameters, game) {
        error!("Failed to create the application: {:?}", err);
        return Err(EngineError::InitializationFailed);
    };
    debug!("Application initialized");

    let platform = fetch_global_application()?.platform.as_ref();

    match renderer_init(&app_name.clone(), platform) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the renderer: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }
    debug!("Renderer initialized");

    unsafe { IS_ENGINE_INITIALIZED = true };

    Ok(())
}

/// Main loop
fn game_loop() -> Result<(), EngineError> {
    let application = fetch_global_application()?;
    match application.run() {
        Ok(()) => Ok(()),
        Err(err) => {
            error!("The application failed to run: {:?}", err);
            Err(EngineError::Unknown)
        }
    }
}

/// Cleanup the engine
fn engine_shutdown() -> Result<(), EngineError> {
    match renderer_shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the renderer: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    }
    debug!("Renderer shutted down");

    if let Err(err) = application_shutdown() {
        error!("Failed to shutdown the application: {:?}", err);
        return Err(EngineError::ShutdownFailed);
    };
    debug!("Application shutted down");

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
    if let Err(err) = engine_init(parameters, game) {
        error!("Failed to initialize the engine: {:?}", err);
        return Err(EngineError::InitializationFailed);
    };
    debug!("Engine initialized");

    // game on start
    let application = fetch_global_application()?;
    if let Err(err) = application.game.on_start() {
        error!(
            "Failed to call the `on_start' method of the game: {:?}",
            err
        );
        return Err(EngineError::InitializationFailed);
    }

    // Game loop
    game_loop()?;

    // game on shutdown
    let application = fetch_global_application()?;
    if let Err(err) = application.game.on_shutdown() {
        error!(
            "Failed to call the `on_shutdown' method of the game: {:?}",
            err
        );
        return Err(EngineError::InitializationFailed);
    }

    // Cleanup
    if let Err(err) = engine_shutdown() {
        error!("Failed to shutdown the engine: {:?}", err);
        return Err(EngineError::ShutdownFailed);
    }

    debug!("Engine shutted down");

    Ok(())
}
