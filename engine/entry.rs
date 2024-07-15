use crate::{
    core::{
        application::{init_application, ApplicationParameters},
        errors::EngineError,
    },
    error,
    game::Game,
};

/// Entry point of the game engine
pub fn start_engine(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<(), EngineError> {
    // Initialization
    let mut application = match init_application(parameters, game) {
        Ok(application) => application,
        Err(err) => {
            error!("Failed to create the application: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    };

    // Game loop.
    match application.run() {
        Ok(()) => (),
        Err(err) => {
            error!("The application failed to run: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    };

    // Cleanup
    match application.shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the application: {:?}", err);
            return Err(EngineError::Unknown);
        }
    };

    match application.game.shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the game: {:?}", err);
            return Err(EngineError::Unknown);
        }
    };

    Ok(())
}
