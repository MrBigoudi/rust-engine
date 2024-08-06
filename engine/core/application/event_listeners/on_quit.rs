use crate::{
    core::{
        application::{fetch_global_application, ApplicationState},
        debug::errors::EngineError,
        systems::events::{EventCode, EventListener},
    },
    error,
};

pub(super) struct ApplicationOnQuitListener;

impl EventListener for ApplicationOnQuitListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        match code {
            EventCode::ApplicationQuit => {
                let app = fetch_global_application()?;
                app.state = ApplicationState::ShuttingDown;
            }
            wrong_code => {
                error!(
                    "Failed to call the application 'OnQuit' listener: got {:?} code",
                    wrong_code
                );
                return Err(EngineError::InvalidValue);
            }
        };

        Ok(true)
    }
}
