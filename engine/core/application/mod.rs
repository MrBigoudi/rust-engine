use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{
    debug, error,
    game::Game,
    platforms::platform::{platform_init, Platform},
    renderer::{renderer_frontend::renderer_draw_frame, renderer_types::RenderFrameData},
};

use super::{debug::clock::Clock, debug::errors::EngineError, systems::input::input_update};

pub mod event_listeners;

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
    pub initial_width: u32,
    pub initial_height: u32,
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
    pub fn initial_width(mut self, width: u32) -> Self {
        self.initial_width = width;
        self
    }
    pub fn initial_height(mut self, height: u32) -> Self {
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
    Suspended,
}

pub(crate) struct Application {
    pub platform: Box<dyn Platform>,
    pub game: Box<dyn Game>,

    pub state: ApplicationState,
    pub clock: Clock,
    pub last_time: f64,
    pub width: u32,
    pub height: u32,
    pub is_resizable: bool,
}

#[derive(Default)]
pub(crate) struct ApplicationWrapper {
    pub application: Option<Application>,
}

unsafe impl Send for Application {}
unsafe impl Sync for Application {}

pub(crate) static mut GLOBAL_APPLICATION: Lazy<Mutex<ApplicationWrapper>> =
    Lazy::new(Mutex::default);

fn fetch_global_application_wrapper(
    error: EngineError,
) -> Result<&'static mut ApplicationWrapper, EngineError> {
    unsafe {
        match GLOBAL_APPLICATION.get_mut() {
            Ok(wrapper) => Ok(wrapper),
            Err(err) => {
                error!("Failed to fetch the global application: {:?}", err);
                Err(error)
            }
        }
    }
}

pub(crate) fn fetch_global_application() -> Result<&'static mut Application, EngineError> {
    let global_application = fetch_global_application_wrapper(EngineError::AccessFailed)?;
    Ok(global_application.application.as_mut().unwrap())
}

pub(crate) fn application_get_framebuffer_size() -> Result<(u32, u32), EngineError> {
    fetch_global_application()?.get_framebuffer_size()
}

/// Shutdown the application
pub(crate) fn application_shutdown() -> Result<(), EngineError> {
    fetch_global_application()?.shutdown()
}

/// Initiate the application
pub(crate) fn application_init(
    parameters: ApplicationParameters,
    game: Box<dyn Game>,
) -> Result<(), EngineError> {
    let platform = platform_init(
        parameters.application_name.clone(),
        parameters.initial_x_position,
        parameters.initial_y_position,
        parameters.initial_width,
        parameters.initial_height,
        parameters.flags.is_window_resizable,
    );

    debug!("Platform initialized");

    let global_application_wrapper =
        fetch_global_application_wrapper(EngineError::InitializationFailed)?;

    let application = match platform {
        Err(err) => {
            error!("Failed to init the platform: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
        Ok(platform) => Application {
            platform: Box::new(platform),
            game,
            state: ApplicationState::Running,
            clock: Clock::default(),
            last_time: 0.,
            width: parameters.initial_width,
            height: parameters.initial_height,
            is_resizable: parameters.flags.is_window_resizable,
        },
    };

    // register events
    if let Err(err) = application.init_event_listener() {
        error!(
            "Failed to initialize the application events listeners: {:?}",
            err
        );
        return Err(EngineError::InitializationFailed);
    }

    global_application_wrapper.application = Some(application);

    Ok(())
}

impl Application {
    pub fn get_framebuffer_size(&self) -> Result<(u32, u32), EngineError> {
        let width = self.width;
        let height = self.height;
        Ok((width, height))
    }

    /// Run the application
    pub fn run(&mut self) -> Result<(), EngineError> {
        self.clock.start(self.platform.as_ref())?;
        self.clock.update(self.platform.as_ref())?;
        self.last_time = self.clock.elapsed_time;

        let mut running_time: f64 = 0.;
        let mut frame_count: u32 = 0;
        let target_frame_seconds: f64 = 1. / 60.;

        'main_loop: while self.state != ApplicationState::ShuttingDown {
            if self.state == ApplicationState::Suspended {
                continue 'main_loop;
            }
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

            // update clock and get delta time.
            self.clock.update(self.platform.as_ref())?;
            let current_time: f64 = self.clock.elapsed_time;
            let delta: f64 = current_time - self.last_time;
            let frame_start_time: f64 = self.platform.as_ref().get_absolute_time_in_seconds()?;

            // update the game
            match self.game.on_update(delta) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to update the game: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }

            // render the game
            match self.game.on_render(delta) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to render the game: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }

            // Create frame and render
            let frame_data = RenderFrameData { delta_time: delta };
            renderer_draw_frame(&frame_data)?;

            // Figure out how long the frame took and, if below
            let frame_end_time: f64 = self.platform.get_absolute_time_in_seconds()?;
            let frame_elapsed_time: f64 = frame_end_time - frame_start_time;
            running_time += frame_elapsed_time;
            let remaining_seconds: f64 = target_frame_seconds - frame_elapsed_time;

            if remaining_seconds > 0. {
                let remaining_ms: u64 = remaining_seconds as u64 * 1000;

                // If there is time left, give it back to the OS.
                let limit_frames = false;
                if remaining_ms > 0 && limit_frames {
                    self.platform.sleep_from_milliseconds(remaining_ms - 1)?;
                }

                frame_count += 1;
            }

            // NOTE: Input update/state copying should always be handled
            // after any input should be recorded; I.E. before this line.
            // As a safety, input is the last thing to be updated before
            // this frame ends.
            match input_update(delta) {
                Ok(()) => (),
                Err(err) => {
                    error!("Failed to update the inputs: {:?}", err);
                    return Err(EngineError::Unknown);
                }
            }

            // debug!("delta: {}, last_time: {}", delta, self.last_time);
            // update last time
            self.last_time = current_time;
        }
        Ok(())
    }

    /// Shutdown the application
    pub fn shutdown(&mut self) -> Result<(), EngineError> {
        self.state = ApplicationState::ShuttingDown;
        match self.platform.shutdown() {
            Err(err) => {
                error!("Failed to shut down the application: {:?}", err);
                Err(EngineError::ShutdownFailed)
            }
            Ok(()) => Ok(()),
        }
    }
}
