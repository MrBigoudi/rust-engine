use engine::{debug, info, warn};
use engine::platform::platform::{init_platform, Platform};

fn main() {
    warn!("Test warning");
    debug!("Test debug");
    info!("Test info");

    let mut platform = init_platform(
        String::from("Test bed"), 
        100, 
        100, 
        1280, 
        720
    ).expect("Failed to init the platform");

    loop {
        let should_quit = platform.handle_events();
        if should_quit {break;}
    }

    platform.shutdown();
}