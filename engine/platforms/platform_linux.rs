use std::ffi::{c_char, CStr};

/// Linux implementation of the platform trait
use xcb::Xid;

use crate::{
    core::{
        debug::errors::EngineError,
        systems::{
            input::{
                keyboard::{intput_process_key, Key, KeyState},
                mouse::{
                    input_process_mouse_button, input_process_mouse_move, MouseButton,
                    MouseButtonState,
                },
            },
            logger::LogLevel,
        },
    },
    error, warn,
};

use super::platform::Platform;

#[derive(Default)]
pub(crate) struct PlatformLinux {
    // Internal state
    pub screen_number: i32,
    pub connection: Option<xcb::Connection>,
    pub screen: Option<xcb::x::ScreenBuf>,
    pub window: Option<xcb::x::Window>,
    pub window_manager_protocols: Option<xcb::x::Atom>,
    pub window_manager_delete_window: Option<xcb::x::Atom>,
    pub key_symbols: Option<*mut xcb_util::ffi::keysyms::xcb_key_symbols_t>,
}

impl Platform for PlatformLinux {
    fn init(
        &mut self,
        window_title: String,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        resizable: bool,
    ) -> Result<(), EngineError> {
        // Connect to the X server
        let (connection, screen_number) = match xcb::Connection::connect(None) {
            Err(err) => {
                error!("Failed to connect to the X server: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
            Ok((connection, screen_number)) => (connection, screen_number),
        };

        self.connection = Some(connection);

        // Fetch the `x::Setup` and get the main `x::Screen` object.
        let setup = self.connection.as_ref().unwrap().get_setup();
        let screen = setup.roots().nth(screen_number as usize).unwrap();

        // Generate an `Xid` for the client window.
        // The type inference is needed here.
        let window: xcb::x::Window = self.connection.as_ref().unwrap().generate_id();

        // We can now create a window. For this we pass a `Request`
        // object to the `send_request_checked` method. The method
        // returns a cookie that will be used to check for success.
        let cookie =
            self.connection
                .as_ref()
                .unwrap()
                .send_request_checked(&xcb::x::CreateWindow {
                    depth: xcb::x::COPY_FROM_PARENT as u8,
                    wid: window,
                    parent: screen.root(),
                    x,
                    y,
                    width,
                    height,
                    border_width: 0, // no border
                    class: xcb::x::WindowClass::InputOutput,
                    visual: screen.root_visual(),
                    // this list must be in same order than `Cw` enum order
                    value_list: &[
                        xcb::x::Cw::BackPixel(screen.black_pixel()),
                        xcb::x::Cw::EventMask(
                            xcb::x::EventMask::EXPOSURE
                                | xcb::x::EventMask::POINTER_MOTION
                                | xcb::x::EventMask::STRUCTURE_NOTIFY
                                | xcb::x::EventMask::KEY_PRESS
                                | xcb::x::EventMask::KEY_RELEASE
                                | xcb::x::EventMask::BUTTON_PRESS
                                | xcb::x::EventMask::BUTTON_RELEASE,
                        ),
                    ],
                });

        // We now check if the window creation worked.
        // A cookie can't be cloned; it is moved to the function.
        if let Err(err) = self.connection.as_ref().unwrap().check_request(cookie) {
            error!("Failed to create the window: {:?}", err);
            return Err(EngineError::InitializationFailed);
        };

        // Let's change the window title
        let cookie =
            self.connection
                .as_ref()
                .unwrap()
                .send_request_checked(&xcb::x::ChangeProperty {
                    mode: xcb::x::PropMode::Replace,
                    window,
                    property: xcb::x::ATOM_WM_NAME,
                    r#type: xcb::x::ATOM_STRING,
                    data: window_title.as_bytes(),
                });
        // And check for success again
        if let Err(err) = self.connection.as_ref().unwrap().check_request(cookie) {
            error!("Failed to set the window title: {:?}", err);
            return Err(EngineError::InitializationFailed);
        };

        if !resizable {
            // TODO:
        }

        // We now show ("map" in X terminology) the window.
        // This time we do not check for success, so we discard the cookie.
        self.connection
            .as_ref()
            .unwrap()
            .send_request(&xcb::x::MapWindow { window });

        // We need a few atoms for our application.
        // We send a few requests in a row and wait for the replies after.
        let (wm_protocols, wm_del_window) = {
            let cookies = (
                self.connection
                    .as_ref()
                    .unwrap()
                    .send_request(&xcb::x::InternAtom {
                        only_if_exists: true,
                        name: b"WM_PROTOCOLS",
                    }),
                self.connection
                    .as_ref()
                    .unwrap()
                    .send_request(&xcb::x::InternAtom {
                        only_if_exists: true,
                        name: b"WM_DELETE_WINDOW",
                    }),
            );
            (
                match self.connection.as_ref().unwrap().wait_for_reply(cookies.0) {
                    Err(err) => {
                        error!("Failed to get the protocols atom: {:?}", err);
                        return Err(EngineError::InitializationFailed);
                    }
                    Ok(reply) => reply.atom(),
                },
                match self.connection.as_ref().unwrap().wait_for_reply(cookies.1) {
                    Err(err) => {
                        error!("Failed to get the delete window atom: {:?}", err);
                        return Err(EngineError::InitializationFailed);
                    }
                    Ok(reply) => reply.atom(),
                },
            )
        };

        self.window_manager_protocols = Some(wm_protocols);
        self.window_manager_delete_window = Some(wm_del_window);

        // We now activate the window close event by sending the following request.
        // If we don't do this we can still close the window by clicking on the "x" button,
        // but the event loop is notified through a connection shutdown error.
        if let Err(err) = self.connection.as_ref().unwrap().check_request(
            self.connection
                .as_ref()
                .unwrap()
                .send_request_checked(&xcb::x::ChangeProperty {
                    mode: xcb::x::PropMode::Replace,
                    window,
                    property: *self.window_manager_protocols.as_mut().unwrap(),
                    r#type: xcb::x::ATOM_ATOM,
                    data: &[self
                        .window_manager_delete_window
                        .as_mut()
                        .unwrap()
                        .resource_id()],
                }),
        ) {
            error!("Failed to activate the window close event: {:?}", err);
            return Err(EngineError::InitializationFailed);
        };

        self.screen_number = screen_number;
        self.screen = Some(screen.to_owned());
        self.window = Some(window);

        // Init keysym
        let key_symbols = unsafe {
            xcb_util::ffi::keysyms::xcb_key_symbols_alloc(
                self.connection.as_ref().unwrap().get_raw_conn() as *mut _,
            )
        };

        if key_symbols.is_null() {
            error!("Failed to allocate key symbols");
            return Err(EngineError::InitializationFailed);
        }

        self.key_symbols = Some(key_symbols);

        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), EngineError> {
        // close the keysym
        unsafe { xcb_util::ffi::keysyms::xcb_key_symbols_free(self.key_symbols.unwrap()) };

        // We close the window
        let window = self.window.unwrap();
        match self.connection.as_ref().unwrap().check_request(
            self.connection
                .as_ref()
                .unwrap()
                .send_request_checked(&xcb::x::DestroyWindow { window }),
        ) {
            Err(err) => {
                error!("Failed to destroy the window: {:?}", err);
                Err(EngineError::InitializationFailed)
            }
            Ok(()) => Ok(()),
        }
    }

    fn get_absolute_time_in_seconds(&self) -> Result<f64, EngineError> {
        match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs_f64()),
            Err(_) => {
                error!("SystemTime before UNIX EPOCH!");
                Err(EngineError::InvalidValue)
            }
        }
    }

    fn sleep_from_milliseconds(&self, ms: u64) -> Result<(), EngineError> {
        let duration_from_milliseconds = std::time::Duration::from_millis(ms);
        std::thread::sleep(duration_from_milliseconds);
        Ok(())
    }

    fn handle_events(&mut self) -> Result<bool, EngineError> {
        let mut quit_flag = false;

        'infinite_loop: loop {
            let event = self.connection.as_ref().unwrap().poll_for_event().unwrap();
            match event {
                // leave loop when no more events to process
                None => break 'infinite_loop,
                Some(event) => {
                    match event {
                        // Input events
                        xcb::Event::Unknown(_) => continue 'infinite_loop,
                        xcb::Event::X(event) => {
                            match event {
                                // Keyboard press / release
                                xcb::x::Event::KeyPress(event) => {
                                    let key_code = event.detail();
                                    let key_mask =
                                        if event.state().contains(xcb::x::KeyButMask::SHIFT) {
                                            1
                                        } else {
                                            0
                                        };
                                    if let Some(key) = self.translate_keycode(key_code, key_mask) {
                                        // debug!("code pressed: {:?}", key);
                                        intput_process_key(key, KeyState::Pressed)?;
                                    };
                                }
                                xcb::x::Event::KeyRelease(event) => {
                                    let key_code = event.detail();
                                    let key_mask =
                                        if event.state().contains(xcb::x::KeyButMask::SHIFT) {
                                            1
                                        } else {
                                            0
                                        };
                                    if let Some(key) = self.translate_keycode(key_code, key_mask) {
                                        // debug!("code release: {:?}", key);
                                        intput_process_key(key, KeyState::Released)?;
                                    };
                                }

                                // Mouse press / release
                                xcb::x::Event::ButtonPress(event) => {
                                    let button = event.detail() as u32;
                                    if button == xcb::x::ButtonIndex::N1 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Left,
                                            MouseButtonState::Pressed,
                                        )?;
                                        // debug!("left button pressed");
                                    } else if button == xcb::x::ButtonIndex::N2 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Middle,
                                            MouseButtonState::Pressed,
                                        )?;
                                        // debug!("middle button pressed");
                                    } else if button == xcb::x::ButtonIndex::N3 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Right,
                                            MouseButtonState::Pressed,
                                        )?;
                                        // debug!("right button pressed");
                                    } else {
                                        warn!("Unknown mouse button: {:?}", button);
                                    };
                                }
                                xcb::x::Event::ButtonRelease(event) => {
                                    let button = event.detail() as u32;
                                    if button == xcb::x::ButtonIndex::N1 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Left,
                                            MouseButtonState::Released,
                                        )?;
                                        // debug!("left button released");
                                    } else if button == xcb::x::ButtonIndex::N2 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Middle,
                                            MouseButtonState::Released,
                                        )?;
                                        // debug!("middle button released");
                                    } else if button == xcb::x::ButtonIndex::N3 as u32 {
                                        input_process_mouse_button(
                                            MouseButton::Right,
                                            MouseButtonState::Released,
                                        )?;
                                        // debug!("right button released");
                                    } else {
                                        warn!("Unknown mouse button: {:?}", button);
                                    };
                                }

                                // Mouse movement
                                xcb::x::Event::MotionNotify(event) => {
                                    // debug!("mouse pos: ({}, {})", event.event_x(), event.event_y());
                                    input_process_mouse_move(event.event_x(), event.event_y())?;
                                }

                                // Resizing
                                xcb::x::Event::ConfigureNotify(_) => {
                                    continue 'infinite_loop; //TODO:
                                }

                                xcb::x::Event::ClientMessage(client_message_event) => {
                                    // Window closing
                                    let message_index_zero = match client_message_event.data() {
                                        xcb::x::ClientMessageData::Data8(data) => data[0] as u32,
                                        xcb::x::ClientMessageData::Data16(data) => data[0] as u32,
                                        xcb::x::ClientMessageData::Data32(data) => data[0],
                                    };
                                    if message_index_zero
                                        == self.window_manager_delete_window.unwrap().resource_id()
                                    {
                                        quit_flag = true;
                                    }
                                }

                                // Other events
                                _ => continue 'infinite_loop, // Ignore other events
                            }
                        }
                    }
                }
            }
        }

        Ok(quit_flag)
    }

    fn console_write(message: &str, log_level: LogLevel) {
        print!(
            "\x1B[{}m{}\x1B[0m",
            PlatformLinux::get_color(log_level),
            message
        );
    }

    fn console_write_error(message: &str, log_level: LogLevel) {
        eprint!(
            "\x1B[{}m{}\x1B[0m",
            PlatformLinux::get_color(log_level),
            message
        );
    }

    fn get_required_extensions(&self) -> Result<Vec<*const i8>, EngineError> {
        let required_extensions_cstr = [
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_KHR_surface\0") },
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_KHR_xcb_surface\0") },
        ];

        let required_extensions: Vec<*const c_char> = required_extensions_cstr
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        Ok(required_extensions)
    }
}

impl PlatformLinux {
    pub fn get_color(log_level: LogLevel) -> &'static str {
        match log_level {
            // https://www.lihaoyi.com/post/BuildyourownCommandLinewithANSIescapecodes.html for other ANSI codes
            LogLevel::Error => "1;31",   // Red foreground
            LogLevel::Warning => "1;33", // Yellow foreground
            LogLevel::Debug => "1;34",   // Blue foreground
            LogLevel::Info => "1;32",    // Green foreground
        }
    }

    // Key translation
    fn translate_keycode(&self, xcb_keycode: u8, col: i32) -> Option<Key> {
        let keysym: u32 = unsafe {
            xcb_util::ffi::keysyms::xcb_key_symbols_get_keysym(
                self.key_symbols.unwrap(),
                xcb_keycode,
                col,
            )
        };

        match keysym {
            0xFF08 => Some(Key::BACKSPACE),
            0xFF0D => Some(Key::ENTER),
            0xFF09 => Some(Key::TAB),
            0xFFE1 => Some(Key::LSHIFT),
            0xFFE2 => Some(Key::RSHIFT),
            0xFFE3 => Some(Key::LCONTROL),
            0xFFE4 => Some(Key::RCONTROL),
            0xFF13 => Some(Key::PAUSE),
            0xFFE5 => Some(Key::CAPITAL),
            0xFF1B => Some(Key::ESCAPE),
            0xFF2C => Some(Key::CONVERT),
            0xFF2D => Some(Key::NONCONVERT),
            0xFF2E => Some(Key::ACCEPT),
            0xFF2F => Some(Key::MODECHANGE),
            0x0020 => Some(Key::SPACE),
            0xFF55 => Some(Key::PRIOR),
            0xFF56 => Some(Key::NEXT),
            0xFF57 => Some(Key::END),
            0xFF50 => Some(Key::HOME),
            0xFF51 => Some(Key::LEFT),
            0xFF52 => Some(Key::UP),
            0xFF53 => Some(Key::RIGHT),
            0xFF54 => Some(Key::DOWN),
            0xFF60 => Some(Key::SELECT),
            0xFF61 => Some(Key::PRINT),
            0xFF62 => Some(Key::EXECUTE),
            // 0xFF63 => Some(Key::SNAPSHOT), // Not supported
            0xFF63 => Some(Key::INSERT),
            0xFFFF => Some(Key::DELETE),
            0xFF6A => Some(Key::HELP),
            0xFFEB => Some(Key::LWIN),
            0xFFEC => Some(Key::RWIN),
            // 0xFF67 => Some(Key::APPS), // Not supported
            // 0xFF65 => Some(Key::SLEEP), // Not supported
            0xFFB0 => Some(Key::NUMPAD0),
            0xFFB1 => Some(Key::NUMPAD1),
            0xFFB2 => Some(Key::NUMPAD2),
            0xFFB3 => Some(Key::NUMPAD3),
            0xFFB4 => Some(Key::NUMPAD4),
            0xFFB5 => Some(Key::NUMPAD5),
            0xFFB6 => Some(Key::NUMPAD6),
            0xFFB7 => Some(Key::NUMPAD7),
            0xFFB8 => Some(Key::NUMPAD8),
            0xFFB9 => Some(Key::NUMPAD9),
            0xFFAA => Some(Key::MULTIPLY),
            0xFFAB => Some(Key::ADD),
            0xFFAC => Some(Key::SEPARATOR),
            0xFFAD => Some(Key::SUBTRACT),
            0xFFAE => Some(Key::DECIMAL),
            0xFFAF => Some(Key::DIVIDE),
            0xFFBE => Some(Key::F1),
            0xFFBF => Some(Key::F2),
            0xFFC0 => Some(Key::F3),
            0xFFC1 => Some(Key::F4),
            0xFFC2 => Some(Key::F5),
            0xFFC3 => Some(Key::F6),
            0xFFC4 => Some(Key::F7),
            0xFFC5 => Some(Key::F8),
            0xFFC6 => Some(Key::F9),
            0xFFC7 => Some(Key::F10),
            0xFFC8 => Some(Key::F11),
            0xFFC9 => Some(Key::F12),
            0xFFCA => Some(Key::F13),
            0xFFCB => Some(Key::F14),
            0xFFCC => Some(Key::F15),
            0xFFCD => Some(Key::F16),
            0xFFCE => Some(Key::F17),
            0xFFCF => Some(Key::F18),
            0xFFD0 => Some(Key::F19),
            0xFFD1 => Some(Key::F20),
            0xFFD2 => Some(Key::F21),
            0xFFD3 => Some(Key::F22),
            0xFFD4 => Some(Key::F23),
            0xFFD5 => Some(Key::F24),
            0xFF7F => Some(Key::NUMLOCK),
            0xFF14 => Some(Key::SCROLL),
            0xFF8D => Some(Key::NUMPADEQUAL),
            0xFFE7 => Some(Key::LMENU),
            0xFFE8 => Some(Key::RMENU),
            0x003B => Some(Key::SEMICOLON),
            0x002B => Some(Key::PLUS),
            0x002C => Some(Key::COMMA),
            0x002D => Some(Key::MINUS),
            0x002E => Some(Key::PERIOD),
            0x002F => Some(Key::SLASH),
            0x0060 => Some(Key::GRAVE),
            0x0041 => Some(Key::A),
            0x0042 => Some(Key::B),
            0x0043 => Some(Key::C),
            0x0044 => Some(Key::D),
            0x0045 => Some(Key::E),
            0x0046 => Some(Key::F),
            0x0047 => Some(Key::G),
            0x0048 => Some(Key::H),
            0x0049 => Some(Key::I),
            0x004A => Some(Key::J),
            0x004B => Some(Key::K),
            0x004C => Some(Key::L),
            0x004D => Some(Key::M),
            0x004E => Some(Key::N),
            0x004F => Some(Key::O),
            0x0050 => Some(Key::P),
            0x0051 => Some(Key::Q),
            0x0052 => Some(Key::R),
            0x0053 => Some(Key::S),
            0x0054 => Some(Key::T),
            0x0055 => Some(Key::U),
            0x0056 => Some(Key::V),
            0x0057 => Some(Key::W),
            0x0058 => Some(Key::X),
            0x0059 => Some(Key::Y),
            0x005A => Some(Key::Z),
            0x0061 => Some(Key::A),
            0x0062 => Some(Key::B),
            0x0063 => Some(Key::C),
            0x0064 => Some(Key::D),
            0x0065 => Some(Key::E),
            0x0066 => Some(Key::F),
            0x0067 => Some(Key::G),
            0x0068 => Some(Key::H),
            0x0069 => Some(Key::I),
            0x006A => Some(Key::J),
            0x006B => Some(Key::K),
            0x006C => Some(Key::L),
            0x006D => Some(Key::M),
            0x006E => Some(Key::N),
            0x006F => Some(Key::O),
            0x0070 => Some(Key::P),
            0x0071 => Some(Key::Q),
            0x0072 => Some(Key::R),
            0x0073 => Some(Key::S),
            0x0074 => Some(Key::T),
            0x0075 => Some(Key::U),
            0x0076 => Some(Key::V),
            0x0077 => Some(Key::W),
            0x0078 => Some(Key::X),
            0x0079 => Some(Key::Y),
            0x007A => Some(Key::Z),
            _ => {
                warn!("Unknown keysym: {:?}", keysym);
                None
            }
        }
    }
}
