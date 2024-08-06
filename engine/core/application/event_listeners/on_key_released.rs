use crate::{
    core::{
        debug::errors::EngineError,
        systems::events::{EventCode, EventListener},
    },
    error,
};

pub(super) struct ApplicationOnKeyReleasedListener;

impl EventListener for ApplicationOnKeyReleasedListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        let key_code = match code {
            EventCode::KeyReleased { key_code } => key_code,
            wrong_code => {
                error!(
                    "Failed to call the application 'OnKeyReleased' listener: got {:?} code",
                    wrong_code
                );
                return Err(EngineError::InvalidValue);
            }
        };
        Ok(false)
    }
}
