use crate::{
    core::{
        debug::errors::EngineError,
        systems::{
            events::{event_fire, EventCode, EventListener},
            input::keyboard::Key,
        },
    },
    error,
};

pub(super) struct ApplicationOnKeyPressedListener;

impl EventListener for ApplicationOnKeyPressedListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        let key_code = match code {
            EventCode::KeyPressed { key_code } => key_code,
            wrong_code => {
                error!(
                    "Failed to call the application 'OnKeyPressed' listener: got {:?} code",
                    wrong_code
                );
                return Err(EngineError::InvalidValue);
            }
        };
        if key_code == (Key::ESCAPE as u16) {
            match event_fire(EventCode::ApplicationQuit) {
                Ok(_) => return Ok(true),
                Err(err) => {
                    error!(
                        "Failed to fire the `{:?}' event",
                        EventCode::ApplicationQuit
                    );
                    return Err(EngineError::Unknown);
                }
            }
        }
        Ok(false)
    }
}
