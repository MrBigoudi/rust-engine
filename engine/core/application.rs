use crate::{
    debug, error,
    game::Game,
    info,
    platforms::platform::{init_platform, Platform},
    warn,
};

use super::{errors::EngineError, logger::init_logger};

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
pub enum ApplicationState {
    Running,
    Suspended,
    ShuttingDown,
}

pub struct Application {
    pub platform: Box<dyn Platform>,
    pub game: Box<dyn Game>,
    pub state: ApplicationState,
}

/// Static variable to allow only a single instantiation of the application
static mut IS_APPLICATION_INITIALIZED: bool = false;

/// Initialize the different subsystems
fn init_subsystems() -> Result<(), EngineError> {
    match init_logger() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the logger: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }
    Ok(())
}

/// Initiate the application
/// Can only be called once
pub fn init_application(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<Application, EngineError> {
    if unsafe { IS_APPLICATION_INITIALIZED } {
        error!("The application is already initialized!");
        return Err(EngineError::MultipleInstantiation);
    }

    match init_subsystems() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the subsystems: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }

    error!("Test error log");
    warn!("Test warn log");
    debug!("Test debug log");
    info!("Test info log");

    let platform = init_platform(
        parameters.application_name.clone(),
        parameters.initial_x_position,
        parameters.initial_y_position,
        parameters.initial_width,
        parameters.initial_height,
        parameters.flags.is_window_resizable,
    );

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

    unsafe { IS_APPLICATION_INITIALIZED = true };
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
