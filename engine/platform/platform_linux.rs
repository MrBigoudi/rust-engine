use crate::{core::logger::LogLevel, error};

use super::platform::Platform;

#[derive(Default)]
pub struct PlatformLinux {
    // Internal state
    pub screen_number: i32, 
    pub connection: Option<xcb::Connection>,
    pub screen: Option<xcb::x::ScreenBuf>,
    pub window: Option<xcb::x::Window>,
    pub window_manager_protocols: Option<xcb::x::Atom>, 
    pub window_manager_delete_window: Option<xcb::x::Atom>,
}

impl PlatformLinux {
    pub fn get_color(log_level: LogLevel) -> &'static str {
        match log_level {
            LogLevel::Error => "0;41",     // Red background
            LogLevel::Warning => "1;33",   // Yellow foreground
            LogLevel::Debug => "1;34",     // Blue foreground
            LogLevel::Info => "1;32",      // Green foreground
        }
    }
}

impl Platform for PlatformLinux {
    fn init(&mut self, window_title: String, x: i16, y: i16, width: u16, height: u16) {
        // Connect to the X server.
        let (connection, screen_number) = xcb::Connection::connect(None)
            .unwrap_or_else(|err| error!("Failed to connect to the X server: {:?}", err));

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
        let cookie = self.connection.as_ref().unwrap().send_request_checked(&xcb::x::CreateWindow {
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
                    | xcb::x::EventMask::BUTTON_RELEASE
                ),
            ],
        });

        // We now check if the window creation worked.
        // A cookie can't be cloned; it is moved to the function.
        self.connection.as_ref().unwrap().check_request(cookie)
            .unwrap_or_else(|err| error!("Failed to create the window: {:?}", err));

        // Let's change the window title
        let cookie = self.connection.as_ref().unwrap().send_request_checked(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window,
            property: xcb::x::ATOM_WM_NAME,
            r#type: xcb::x::ATOM_STRING,
            data: window_title.as_bytes(),
        });
        // And check for success again
        self.connection.as_ref().unwrap().check_request(cookie)
            .unwrap_or_else(|err| error!("Failed to set the window title: {:?}", err));

        // We now show ("map" in X terminology) the window.
        // This time we do not check for success, so we discard the cookie.
        self.connection.as_ref().unwrap().send_request(&xcb::x::MapWindow {
            window,
        });

        // We need a few atoms for our application.
        // We send a few requests in a row and wait for the replies after.
        let (wm_protocols, wm_del_window) = {
            let cookies = (
                self.connection.as_ref().unwrap().send_request(&xcb::x::InternAtom {
                    only_if_exists: true,
                    name: b"WM_PROTOCOLS",
                }),
                self.connection.as_ref().unwrap().send_request(&xcb::x::InternAtom {
                    only_if_exists: true,
                    name: b"WM_DELETE_WINDOW",
                }),
            );
            (
                self.connection.as_ref().unwrap().wait_for_reply(cookies.0).unwrap().atom(),
                self.connection.as_ref().unwrap().wait_for_reply(cookies.1).unwrap().atom(),
            )
        };

        self.window_manager_protocols = Some(wm_protocols);
        self.window_manager_delete_window = Some(wm_del_window);

        // We now activate the window close event by sending the following request.
        // If we don't do this we can still close the window by clicking on the "x" button,
        // but the event loop is notified through a connection shutdown error.
        self.connection.as_ref().unwrap().check_request(self.connection.as_ref().unwrap().send_request_checked(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window,
            property: *self.window_manager_protocols.as_mut().unwrap(),
            r#type: xcb::x::ATOM_ATOM,
            data: &[*self.window_manager_delete_window.as_mut().unwrap()],
        })).unwrap_or_else(|err| error!("Failed to activate the window close event: {:?}", err));

        self.screen_number = screen_number;
        self.screen = Some(screen.to_owned());
        self.window = Some(window);
    }

    fn shutdown(&mut self) {
        // We close the window
        let window = self.window.unwrap();
        self.connection.as_ref().unwrap().check_request(
            self.connection.as_ref().unwrap().send_request_checked(&xcb::x::DestroyWindow {window})
            ).unwrap_or_else(
                |err| error!("Failed to destroy the window: {:?}", err)
        );
    }
    
    fn get_absolute_time_in_seconds(&mut self) -> f64 {
        match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs_f64(),
            Err(_) => error!("SystemTime before UNIX EPOCH!"),
        }
    }
    
    fn sleep_from_milliseconds(&mut self, ms: u64){
        let duration_from_milliseconds = std::time::Duration::from_millis(ms);
        std::thread::sleep(duration_from_milliseconds);
    }

    fn handle_events(&mut self) -> bool {
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
                                xcb::x::Event::KeyPress(_) => continue 'infinite_loop, //TODO:,
                                xcb::x::Event::KeyRelease(_) => continue 'infinite_loop, //TODO:

                                // Mouse press / release
                                xcb::x::Event::ButtonPress(_) => continue 'infinite_loop, //TODO:
                                xcb::x::Event::ButtonRelease(_) => continue 'infinite_loop, //TODO:

                                // Mouse movement
                                xcb::x::Event::MotionNotify(_) => continue 'infinite_loop, //TODO:

                                // Resizing
                                xcb::x::Event::ConfigureNotify(_) => continue 'infinite_loop, //TODO:

                                xcb::x::Event::ClientMessage(client_message_event) => {
                                    // Window closing
                                    if client_message_event.r#type() == self.window_manager_delete_window.unwrap() {
                                        quit_flag = true;
                                    }
                                }

                                // Other events
                                _ => continue 'infinite_loop, // Ignore other events
                            }
                        },
                    }
                },
            }
        }

        quit_flag
    }
    
    fn console_write(message: &str, log_level: LogLevel) {
        print!("\x1B[{}m{}\x1B[0m", PlatformLinux::get_color(log_level), message);
    }

    fn console_write_error(message: &str, log_level: LogLevel) {
        eprint!("\x1B[{}m{}\x1B[0m", PlatformLinux::get_color(log_level), message);
    }
}