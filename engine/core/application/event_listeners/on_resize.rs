use crate::{
    core::{
        application::{fetch_global_application, ApplicationState},
        debug::errors::EngineError,
        systems::events::{EventCode, EventListener},
    },
    error, info,
    renderer::renderer_frontend::fetch_global_renderer,
};

pub(super) struct ApplicationOnResizedListener;

impl EventListener for ApplicationOnResizedListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        let app = fetch_global_application()?;
        if !app.is_resizable {
            return Ok(true);
        }

        let (width, height) = match code {
            EventCode::Resized { width, height } => (width, height),
            wrong_code => {
                error!(
                    "Failed to call the application 'OnResize' listener: got {:?} code",
                    wrong_code
                );
                return Err(EngineError::InvalidValue);
            }
        };

        let app = fetch_global_application()?;
        let old_with = app.width;
        let old_height = app.height;
        if old_with != width || old_height != height {
            app.width = width;
            app.height = height;

            info!(
                "Window resized, new size: width={:?}, height={:?}",
                width, height
            );

            // Minimization
            if width == 0 || height == 0 {
                info!("Window minimized, suspending the application");
                app.state = ApplicationState::Suspended;
                return Ok(true);
            }

            // Quit suspended mode
            if app.state == ApplicationState::Suspended {
                info!("Window restored, resuming the application");
                app.state = ApplicationState::Running;
            }

            // game on resize
            if let Err(err) = app.game.on_resize(width, height) {
                error!(
                    "Failed to call the `on_resize' function of the game: {:?}",
                    err
                );
                return Err(EngineError::UpdateFailed);
            }
            let renderer = fetch_global_renderer(EngineError::UpdateFailed)?;
            renderer.resize(width, height)?;
        }

        Ok(true)
    }
}
