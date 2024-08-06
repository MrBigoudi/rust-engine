use std::sync::{Arc, Mutex};

use on_key_pressed::ApplicationOnKeyPressedListener;
use on_key_released::ApplicationOnKeyReleasedListener;
use on_quit::ApplicationOnQuitListener;
use on_resize::ApplicationOnResizedListener;

use crate::{
    core::{
        debug::errors::EngineError,
        systems::events::{event_register, EventCode, EventListener},
    },
    error,
};

use super::Application;

pub mod on_key_pressed;
pub mod on_key_released;
pub mod on_quit;
pub mod on_resize;

impl Application {
    pub(super) fn init_event_listener(&self) -> Result<(), EngineError> {
        let on_quit_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnQuitListener {}));
        let on_key_pressed_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnKeyPressedListener {}));
        let on_key_released_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnKeyReleasedListener {}));
        let on_resized_listener: Arc<Mutex<dyn EventListener>> =
            Arc::new(Mutex::new(ApplicationOnResizedListener {}));

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

        if let Err(err) = event_register(EventCode::any_resized(), Arc::clone(&on_resized_listener))
        {
            error!("Failed to register the `Resized' event listener: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }

        Ok(())
    }
}
