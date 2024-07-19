use std::sync::Mutex;

use keyboard::{Key, KeyState, KeyboardState};
use mouse::{MouseButton, MouseButtonState, MouseState};
use once_cell::sync::Lazy;

use crate::{core::debug::errors::EngineError, error};

pub mod keyboard;
pub mod mouse;

#[derive(Default)]
pub(crate) struct InputState {
    pub is_initialized: bool,
    pub keyboard_current_state: KeyboardState,
    pub keyboard_previous_state: KeyboardState,
    pub mouse_current_state: MouseState,
    pub mouse_previous_state: MouseState,
}

impl InputState {
    pub fn get_current_key_state(&self, key: Key) -> KeyState {
        self.keyboard_current_state.keys[key as usize]
    }

    pub fn get_previous_key_state(&self, key: Key) -> KeyState {
        self.keyboard_previous_state.keys[key as usize]
    }

    pub fn get_current_mouse_button_state(&self, mouse_button: MouseButton) -> MouseButtonState {
        self.mouse_current_state.buttons[mouse_button as usize]
    }

    pub fn get_previous_mouse_button_state(&self, mouse_button: MouseButton) -> MouseButtonState {
        self.mouse_previous_state.buttons[mouse_button as usize]
    }

    pub fn get_current_mouse_position(&self) -> (i16, i16) {
        (self.mouse_current_state.x, self.mouse_current_state.y)
    }

    pub fn get_previous_mouse_position(&self) -> (i16, i16) {
        (self.mouse_previous_state.x, self.mouse_previous_state.y)
    }

    pub fn set_current_mouse_position(&mut self, x: i16, y: i16) {
        self.mouse_current_state.x = x;
        self.mouse_current_state.y = y;
    }

    pub fn set_previous_mouse_position(&mut self, x: i16, y: i16) {
        self.mouse_previous_state.x = x;
        self.mouse_previous_state.y = y;
    }

    pub fn set_current_key_state(&mut self, key: Key, state: KeyState) {
        self.keyboard_current_state.keys[key as usize] = state;
    }

    pub fn set_previous_key_state(&mut self, key: Key, state: KeyState) {
        self.keyboard_previous_state.keys[key as usize] = state;
    }

    pub fn set_current_mouse_button_state(
        &mut self,
        mouse_button: MouseButton,
        state: MouseButtonState,
    ) {
        self.mouse_current_state.buttons[mouse_button as usize] = state;
    }

    pub fn set_previous_mouse_button_state(
        &mut self,
        mouse_button: MouseButton,
        state: MouseButtonState,
    ) {
        self.mouse_previous_state.buttons[mouse_button as usize] = state;
    }
}

/// Initiate the engine input subsystem
pub(crate) fn input_init() -> Result<(), EngineError> {
    let global_state = fetch_global_input_state(EngineError::InitializationFailed)?;
    global_state.is_initialized = true;
    Ok(())
}

/// Shutdown the engine input subsystem
pub(crate) fn input_shutdown() -> Result<(), EngineError> {
    unsafe { GLOBAL_INPUT_STATE = Lazy::new(Mutex::default) };
    Ok(())
}

/// Update the engine input subsystem
pub(crate) fn input_update(_delta_time: f64) -> Result<(), EngineError> {
    let global_state = fetch_global_input_state(EngineError::Unknown)?;
    // copy current states to previous states
    global_state.keyboard_previous_state = global_state.keyboard_current_state;
    global_state.mouse_previous_state = global_state.mouse_current_state;
    Ok(())
}

pub(crate) static mut GLOBAL_INPUT_STATE: Lazy<Mutex<InputState>> = Lazy::new(Mutex::default);

fn fetch_global_input_state(error: EngineError) -> Result<&'static mut InputState, EngineError> {
    unsafe {
        match GLOBAL_INPUT_STATE.get_mut() {
            Ok(state) => Ok(state),
            Err(err) => {
                error!("Failed to fetch the global input state: {:?}", err);
                Err(error)
            }
        }
    }
}
