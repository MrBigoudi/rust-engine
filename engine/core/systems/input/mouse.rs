use crate::{
    core::{
        errors::EngineError,
        systems::events::{event_fire, EventCode},
    },
    error,
};

use super::fetch_global_input_state;

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

const NUMBER_OF_MOUSE_BUTTONS: usize = 3;

#[derive(Clone, Copy)]
pub(crate) struct MouseState {
    pub x: i16,
    pub y: i16,
    pub buttons: [MouseButtonState; NUMBER_OF_MOUSE_BUTTONS],
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            buttons: [MouseButtonState::Released; NUMBER_OF_MOUSE_BUTTONS],
        }
    }
}

impl MouseButton {
    pub fn get_current_state(&self) -> Result<MouseButtonState, EngineError> {
        let global_state = fetch_global_input_state(EngineError::Unknown)?;
        if !global_state.is_initialized {
            error!("Failed to get the current state of the mouse button `{:?}':\nthe global input state is not initialized", self);
            return Err(EngineError::NotInitialized);
        }
        Ok(global_state.get_current_mouse_button_state(*self))
    }

    pub fn get_previous_state(&self) -> Result<MouseButtonState, EngineError> {
        let global_state = fetch_global_input_state(EngineError::Unknown)?;
        if !global_state.is_initialized {
            error!("Failed to get the previous state of the mouse button `{:?}':\nthe global input state is not initialized", self);
            return Err(EngineError::NotInitialized);
        }
        Ok(global_state.get_previous_mouse_button_state(*self))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButtonState {
    Pressed,
    Released,
}

pub fn intput_get_mouse_position() -> Result<(i16, i16), EngineError> {
    let global_state = fetch_global_input_state(EngineError::Unknown)?;
    if !global_state.is_initialized {
        error!(
            "Failed to get the current mouse position:\nthe global input state is not initialized"
        );
        return Err(EngineError::NotInitialized);
    }
    Ok(global_state.get_current_mouse_position())
}

pub fn intput_get_mouse_previous_position() -> Result<(i16, i16), EngineError> {
    let global_state = fetch_global_input_state(EngineError::Unknown)?;
    if !global_state.is_initialized {
        error!(
            "Failed to get the previous mouse position:\nthe global input state is not initialized"
        );
        return Err(EngineError::NotInitialized);
    }
    Ok(global_state.get_previous_mouse_position())
}

/// Process a mouse
pub(crate) fn input_process_mouse_button(
    button: MouseButton,
    state: MouseButtonState,
) -> Result<(), EngineError> {
    let global_state = fetch_global_input_state(EngineError::Unknown)?;
    // handle if the state changed
    if global_state.get_current_mouse_button_state(button) != state {
        // update internal state
        global_state.set_current_mouse_button_state(button, state);

        // fire an event
        let code = match state {
            MouseButtonState::Pressed => EventCode::MouseButtonPressed {
                button: button as u16,
            },
            MouseButtonState::Released => EventCode::MouseButtonReleased {
                button: button as u16,
            },
        };
        event_fire(code)?;
    }

    Ok(())
}

pub(crate) fn input_process_mouse_move(x: i16, y: i16) -> Result<(), EngineError> {
    let global_state = fetch_global_input_state(EngineError::Unknown)?;
    // handle if the state changed
    if global_state.get_current_mouse_position() != (x, y) {
        // update internal state
        global_state.set_current_mouse_position(x, y);

        // fire an event
        event_fire(EventCode::MouseMoved { x, y })?;
    }

    Ok(())
}

pub(crate) fn input_process_mouse_wheel(z_delta: i8) -> Result<(), EngineError> {
    // fire an event
    event_fire(EventCode::MouseWheel { z_delta })?;
    Ok(())
}
