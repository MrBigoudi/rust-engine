use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    core::{
        application::ApplicationState,
        errors::EngineError,
        systems::{
            events::{event_fire, event_register, EventCode, EventListener},
            input::keyboard::Key,
        },
    },
    debug, error,
};

use super::{Application, ApplicationInternalState};

struct ApplicationOnQuitListener {
    pub internal_state: Arc<Mutex<ApplicationInternalState>>,
}
impl ApplicationOnQuitListener {
    fn get_internal_state(&self) -> Result<MutexGuard<ApplicationInternalState>, EngineError> {
        match self.internal_state.lock() {
            Ok(state) => Ok(state),
            Err(err) => {
                error!("Failed to get the application internal state for the OnQuitEvent listener: {:?}", err);
                Err(EngineError::Synchronisation)
            }
        }
    }
}
impl EventListener for ApplicationOnQuitListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        debug!("quit callback");
        self.get_internal_state()?.state = ApplicationState::ShuttingDown;
        Ok(true)
    }
}
struct ApplicationOnKeyPressedListener {
    pub internal_state: Arc<Mutex<ApplicationInternalState>>,
}
impl EventListener for ApplicationOnKeyPressedListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        let key_code = match code {
            EventCode::KeyPressed { key_code } => key_code,
            _ => return Err(EngineError::InvalidValue),
        };
        debug!("key callback");
        if key_code == (Key::ESCAPE as u16) {
            debug!("Quit event fired");
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
impl ApplicationOnKeyPressedListener {
    fn get_internal_state(&self) -> Result<MutexGuard<ApplicationInternalState>, EngineError> {
        match self.internal_state.lock() {
            Ok(state) => Ok(state),
            Err(err) => {
                error!("Failed to get the application internal state for the OnKeyPressedEvent listener: {:?}", err);
                Err(EngineError::Synchronisation)
            }
        }
    }
}
struct ApplicationOnKeyReleasedListener {
    pub internal_state: Arc<Mutex<ApplicationInternalState>>,
}
impl ApplicationOnKeyReleasedListener {
    fn get_internal_state(&self) -> Result<MutexGuard<ApplicationInternalState>, EngineError> {
        match self.internal_state.lock() {
            Ok(state) => Ok(state),
            Err(err) => {
                error!("Failed to get the application internal state for the OnKeyReleasedEvent listener: {:?}", err);
                Err(EngineError::Synchronisation)
            }
        }
    }
}
impl EventListener for ApplicationOnKeyReleasedListener {
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError> {
        let key_code = match code {
            EventCode::KeyReleased { key_code } => key_code,
            _ => return Err(EngineError::InvalidValue),
        };
        debug!("key released callback: {:?}", key_code);
        Ok(false)
    }
}

impl Application {
    pub(super) fn init_event_listener(&self) -> Result<(), EngineError> {
        let on_quit_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnQuitListener {
                internal_state: Arc::clone(&self.internal_state),
            }));
        let on_key_pressed_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnKeyPressedListener {
                internal_state: Arc::clone(&self.internal_state),
            }));
        let on_key_released_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnKeyReleasedListener {
                internal_state: Arc::clone(&self.internal_state),
            }));

        if let Err(err) = event_register(EventCode::ApplicationQuit, Arc::clone(&on_quit_listener))
        {
            error!(
                "Failed to register the `ApplicationQuit' event listener: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        if let Err(err) = event_register(
            EventCode::any_key_pressed(),
            Arc::clone(&on_key_pressed_listener),
        ) {
            error!(
                "Failed to register the `KeyPressed' event listener: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        if let Err(err) = event_register(
            EventCode::any_key_released(),
            Arc::clone(&on_key_released_listener),
        ) {
            error!(
                "Failed to register the `KeyReleased' event listener: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        Ok(())
    }
}
