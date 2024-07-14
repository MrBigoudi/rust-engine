use engine::platforms::platform::{init_platform, Platform};

fn main() {
    let mut platform = init_platform(String::from("Test bed"), 100, 100, 1280, 720, false)
        .expect("Failed to init the platform");

    loop {
        let should_quit = platform.handle_events();
        if should_quit {
            break;
        }
    }

    platform.shutdown();
}
