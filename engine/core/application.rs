use crate::{
    debug, error,
    game::Game,
    platforms::platform::{platform_init, Platform},
};

use super::{errors::EngineError, systems::input::input_update};

/// Flags for the application
pub struct ApplicationParametersFlags {
    /// Enable window resizing, default to true
    pub is_window_resizable: bool,
    /// Center the window, default to false
    pub is_window_centered: bool,
}

impl ApplicationParametersFlags {
    pub fn is_window_resizable(mut self, flag: bool) -> Self {
        self.is_window_resizable = flag;
        self
    }
    pub fn is_window_centered(mut self, flag: bool) -> Self {
        self.is_window_centered = flag;
        self
    }
}

impl Default for ApplicationParametersFlags {
    fn default() -> Self {
        Self {
            is_window_resizable: true,
            is_window_centered: false,
        }
    }
}

/// The application's parameters
pub struct ApplicationParameters {
    pub application_name: String,
    pub initial_x_position: i16,
    pub initial_y_position: i16,
    pub initial_width: u16,
    pub initial_height: u16,
    pub flags: ApplicationParametersFlags,
}

impl ApplicationParameters {
    pub fn initial_x_position(mut self, x: i16) -> Self {
        self.initial_x_position = x;
        self
    }
    pub fn initial_y_position(mut self, y: i16) -> Self {
        self.initial_y_position = y;
        self
    }
    pub fn initial_width(mut self, width: u16) -> Self {
        self.initial_width = width;
        self
    }
    pub fn initial_height(mut self, height: u16) -> Self {
        self.initial_height = height;
        self
    }
    pub fn application_name(mut self, name: String) -> Self {
        self.application_name = name;
        self
    }
}

impl Default for ApplicationParameters {
    fn default() -> Self {
        Self {
            application_name: String::from("NewApp"),
            initial_x_position: 100,
            initial_y_position: 100,
            initial_width: 1280,
            initial_height: 720,
            flags: Default::default(),
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum ApplicationState {
    Running,
    ShuttingDown,
}

pub(crate) struct Application {
    pub platform: Box<dyn Platform>,
    pub game: Box<dyn Game>,
    pub state: ApplicationState,
}

/// Initiate the application
pub(crate) fn application_init(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<Application, EngineError> {
    let platform = platform_init(
        parameters.application_name.clone(),
        parameters.initial_x_position,
        parameters.initial_y_position,
        parameters.initial_width,
        parameters.initial_height,
        parameters.flags.is_window_resizable,
    );

    debug!("Platform initialized");

    let application = match platform {
        Err(err) => {
            error!("Failed to init the platform: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
        Ok(platform) => Application {
            platform: Box::new(platform),
            state: ApplicationState::Running,
            game,
        },
    };

    Ok(application)
}

impl Application {
    /// Run the application
    pub fn run(&mut self) -> Result<(), EngineError> {
        'main_loop: while self.state == ApplicationState::Running {
            // handle the events
            let should_quit = match self.platform.handle_events() {
                Ok(flag) => flag,
                Err(err) => {
                    error!(
                        "The application encountered an issue while running: {:?}",
                        err
                    );
                    return Err(EngineError::Unknown);
                }
            };
            if should_quit {
                break 'main_loop;
            }

            // update the game
            match self.game.update(0.) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to update the game: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }

            // render the game
            match self.game.render(0.) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to render the game: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }

            // NOTE: Input update/state copying should always be handled
            // after any input should be recorded; I.E. before this line.
            // As a safety, input is the last thing to be updated before
            // this frame ends.
            match input_update(0.) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to update the inputs: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }
        }
        Ok(())
    }

    /// Shutdown the application
    pub fn shutdown(&mut self) -> Result<(), EngineError> {
        self.state = ApplicationState::ShuttingDown;
        match self.platform.shutdown() {
            Err(err) => {
                error!("Failed to shut down the application: {:?}", err);
                Err(EngineError::CleaningFailed)
            }
            Ok(()) => Ok(()),
        }
    }
}
