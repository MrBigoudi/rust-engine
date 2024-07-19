use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use crate::{core::errors::EngineError, error, warn};

/// System internal event codes
#[derive(Clone, Copy, Debug)]
pub(crate) enum EventCode {
    /// Shuts the application down on the next frame
    ApplicationQuit,
    /// Keyboard key pressed
    KeyPressed { key_code: u16 },
    /// Keyboard key released
    KeyReleased { key_code: u16 },
    /// Mouse button pressed
    MouseButtonPressed { button: u16 },
    /// Mouse button released
    MouseButtonReleased { button: u16 },
    /// Mouse moved
    MouseMoved { x: i16, y: i16 },
    /// Mouse wheel moved
    MouseWheel { z_delta: i8 },
    /// Resized/resolution changed from the OS
    Resized { width: u16, height: u16 },
}

impl EventCode {
    pub fn any_key_pressed() -> Self {
        EventCode::KeyPressed { key_code: 0 }
    }
    pub fn any_key_released() -> Self {
        EventCode::KeyReleased { key_code: 0 }
    }
    pub fn any_mouse_button_pressed() -> Self {
        EventCode::MouseButtonPressed { button: 0 }
    }
    pub fn any_mouse_button_released() -> Self {
        EventCode::MouseButtonReleased { button: 0 }
    }
    pub fn any_mouse_moved() -> Self {
        EventCode::MouseMoved { x: 0, y: 0 }
    }
    pub fn any_mouse_wheel() -> Self {
        EventCode::MouseWheel { z_delta: 0 }
    }
    pub fn any_resized() -> Self {
        EventCode::Resized {
            width: 0,
            height: 0,
        }
    }
}

pub(crate) const NUMBER_OF_EVENT_CODES: usize = 8;

pub(crate) trait EventListener {
    /// Callback to be called when an event is received
    /// Return true if don't want any other listener to handle the event
    fn on_event_callback(&mut self, code: EventCode) -> Result<bool, EngineError>;
}

/// Register to listen for when events are sent with the provided code
pub(crate) fn event_register(
    code: EventCode,
    listener: Arc<Mutex<dyn EventListener>>,
) -> Result<(), EngineError> {
    let global_events_system = match fetch_global_events(EngineError::Unknown) {
        Ok(events_system) => events_system,
        Err(err) => {
            error!("Failed to register the event");
            return Err(err);
        }
    };
    global_events_system.event_register(code, listener)
}

/// Register to listen for when events are sent with the provided code
pub(crate) fn event_unregister(
    code: EventCode,
    listener: Arc<Mutex<dyn EventListener>>,
) -> Result<(), EngineError> {
    let global_events_system = match fetch_global_events(EngineError::Unknown) {
        Ok(events_system) => events_system,
        Err(err) => {
            error!("Failed to unregister the event");
            return Err(err);
        }
    };
    global_events_system.event_unregister(code, listener)
}

/// Fires an event to listeners of the given code
pub(crate) fn event_fire(code: EventCode) -> Result<(), EngineError> {
    let global_events_system = match fetch_global_events(EngineError::Unknown) {
        Ok(events_system) => events_system,
        Err(err) => {
            error!("Failed to fire the event");
            return Err(err);
        }
    };
    global_events_system.event_fire(code)
}

pub(crate) struct EventListenerRegistered {
    listener: Arc<Mutex<dyn EventListener>>,
}

impl PartialEq for EventListenerRegistered {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(
            self.listener.as_ref() as *const _,
            other.listener.as_ref() as *const _,
        )
    }
}

/// The engine event system
#[derive(Default)]
pub(crate) struct EventSystem {
    pub is_initialized: bool,
    /// Lookup table for event codes
    pub lookup_table: [Vec<EventListenerRegistered>; NUMBER_OF_EVENT_CODES],
}

impl EventSystem {
    pub fn get_lookup_table_index(code: EventCode) -> usize {
        match code {
            EventCode::ApplicationQuit => 0,
            EventCode::KeyPressed { key_code: _ } => 1,
            EventCode::KeyReleased { key_code: _ } => 2,
            EventCode::MouseButtonPressed { button: _ } => 3,
            EventCode::MouseButtonReleased { button: _ } => 4,
            EventCode::MouseMoved { x: _, y: _ } => 5,
            EventCode::MouseWheel { z_delta: _ } => 6,
            EventCode::Resized {
                width: _,
                height: _,
            } => 7,
        }
    }

    /// Register to listen for when events are sent with the provided code
    pub fn event_register(
        &mut self,
        code: EventCode,
        listener: Arc<Mutex<dyn EventListener>>,
    ) -> Result<(), EngineError> {
        if !self.is_initialized {
            let err = EngineError::NotInitialized;
            error!("The events system is not initialized : {:?}", err);
            return Err(err);
        }
        let listener_to_register = EventListenerRegistered { listener };
        let registered_listeners =
            &mut self.lookup_table[EventSystem::get_lookup_table_index(code)];
        if !registered_listeners.contains(&listener_to_register) {
            registered_listeners.push(listener_to_register);
        } else {
            let err = EngineError::Duplicate;
            warn!(
                "The listener is already registered for this event: {:?}",
                err
            );
        }
        Ok(())
    }

    /// Register to listen for when events are sent with the provided code
    pub fn event_unregister(
        &mut self,
        code: EventCode,
        listener: Arc<Mutex<dyn EventListener>>,
    ) -> Result<(), EngineError> {
        if !self.is_initialized {
            let err = EngineError::NotInitialized;
            error!("The events system is not initialized : {:?}", err);
            return Err(err);
        }
        let listener_to_register = EventListenerRegistered { listener };
        let registered_listeners =
            &mut self.lookup_table[EventSystem::get_lookup_table_index(code)];
        registered_listeners
            .retain(|registered_listeners| !registered_listeners.eq(&listener_to_register));
        Ok(())
    }

    /// Fires an event to listeners of the given code
    pub fn event_fire(&mut self, code: EventCode) -> Result<(), EngineError> {
        let registered_listeners =
            &mut self.lookup_table[EventSystem::get_lookup_table_index(code)];
        for registered_listener in registered_listeners {
            let listener_lock = registered_listener.listener.lock();
            if let Ok(mut listener) = listener_lock {
                match listener.on_event_callback(code) {
                    Ok(keep_handling) => {
                        if !keep_handling {
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        error!("Failed to run the listener callback: {:?}", err);
                        return Err(err);
                    }
                }
                // MutexGuard listener is dropped here, releasing the lock
            } else {
                // Handle case where lock cannot be acquired
                warn!("Failed to acquire lock for listener");
                return Err(EngineError::Synchronisation);
            }
        }
        Ok(())
    }
}

pub(crate) static mut GLOBAL_EVENTS: Lazy<Mutex<EventSystem>> = Lazy::new(Mutex::default);

fn fetch_global_events(error: EngineError) -> Result<&'static mut EventSystem, EngineError> {
    unsafe {
        match GLOBAL_EVENTS.get_mut() {
            Ok(events) => Ok(events),
            Err(err) => {
                error!("Failed to fetch the global events table: {:?}", err);
                Err(error)
            }
        }
    }
}

/// Initiate the engine events
pub(crate) fn events_init() -> Result<(), EngineError> {
    let global_events = fetch_global_events(EngineError::InitializationFailed)?;
    global_events.lookup_table = Default::default();
    global_events.is_initialized = true;
    Ok(())
}

/// Shutdown the engine events
pub(crate) fn events_shutdown() -> Result<(), EngineError> {
    unsafe {
        if let Ok(mut global_events) = GLOBAL_EVENTS.lock() {
            for vec in &mut global_events.lookup_table {
                vec.clear();
            }
        }
        // Empty GLOBAL_EVENTS
        GLOBAL_EVENTS = Lazy::new(Mutex::default);
    }
    Ok(())
}
