use engine::{
    core::{
        debug::errors::EngineError,
        systems::input::{input_is_key_down, input_is_key_up, keyboard::Key},
    }, error, game::Game, renderer::renderer_frontend::renderer_set_main_camera,
};

use super::camera::{CameraMovement, MovementDirection};

#[derive(Default)]
pub struct TestBedGame {
    pub camera: CameraMovement,
}

impl TestBedGame {
    fn handle_input_camera(&mut self, delta_time: f64) -> Result<(), EngineError> {
        // move camera
        if input_is_key_down(Key::W).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Forward, delta_time);
        }
        if input_is_key_down(Key::S).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Backward, delta_time);
        }
        if input_is_key_down(Key::A).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Left, delta_time);
        }
        if input_is_key_down(Key::D).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Right, delta_time);
        }
        if input_is_key_down(Key::UP).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Up, delta_time);
        }
        if input_is_key_down(Key::DOWN).unwrap() {
            self.camera
                .handle_movement(MovementDirection::Down, delta_time);
        }
        // handle acceleration
        if input_is_key_down(Key::LSHIFT).unwrap() {
            self.camera.is_accelerating = true;
        }
        if input_is_key_up(Key::LSHIFT).unwrap() {
            self.camera.is_accelerating = false;
        }
        Ok(())
    }

    fn handle_input(&mut self, delta_time: f64) -> Result<(), EngineError> {
        if let Err(err) = self.handle_input_camera(delta_time) {
            error!("Failed to handle input in the testbed game: {:?}", err);
            return Err(EngineError::Unknown);
        }
        Ok(())
    }
}

impl Game for TestBedGame {
    fn on_start(&mut self) -> Result<(), EngineError> {
        self.camera = CameraMovement::new()?;
        Ok(())
    }

    fn on_update(&mut self, delta_time: f64) -> Result<(), EngineError> {
        if let Err(err) = self.handle_input(delta_time) {
            error!("Failed to handle input in the testbed game: {:?}", err);
            return Err(EngineError::Unknown);
        }
        renderer_set_main_camera(&self.camera.camera)?;

        Ok(())
    }
}
