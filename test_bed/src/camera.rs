use std::f32::consts::PI;

use engine::{
    core::debug::errors::EngineError,
    renderer::{renderer_frontend::renderer_get_main_camera, scene::camera::Camera},
};

pub enum MovementDirection {
    Forward,
    Backward,
    Up,
    Down,
    Left,
    Right,
}

pub struct CameraMovement {
    pub camera: Camera,
    // movement attributes
    pub right: glam::Vec3,
    pub world_up: glam::Vec3,
    pub acceleration: f32,
    pub speed: f32,
    pub yaw_rad: f32,
    pub pitch_rad: f32,
    pub is_accelerating: bool,
}

impl Default for CameraMovement {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            right: glam::Vec3::new(1.0, 0.0, 0.0),
            world_up: glam::Vec3::new(0.0, 1.0, 0.0),
            acceleration: 5.0,
            speed: 20.0,
            yaw_rad: 3.0 * PI / 4.0,
            pitch_rad: 0.0,
            is_accelerating: false,
        }
    }
}

impl CameraMovement {
    pub fn new() -> Result<Self, EngineError> {
        let mut camera = Self {
            camera: renderer_get_main_camera()?,
            ..Default::default()
        };
        camera.update();
        Ok(camera)
    }

    fn update(&mut self) {
        // calculate the new center vector
        let front = glam::Vec3::new(
            self.yaw_rad.cos() * self.pitch_rad.cos(),
            self.pitch_rad.sin(),
            self.yaw_rad.sin() * self.pitch_rad.cos(),
        );
        self.camera.center = front.normalize();
        // also re-calculate the Right and Up vector
        self.right = self.camera.center.cross(self.world_up).normalize();
        self.camera.up = self.right.cross(self.camera.center);
        self.camera.update_view();
    }

    pub fn handle_movement(&mut self, direction: MovementDirection, delta_time: f64) {
        let mut velocity = self.speed * delta_time as f32;
        if self.is_accelerating {
            velocity *= self.acceleration;
        }
        match direction {
            MovementDirection::Forward => self.camera.eye -= self.camera.center * velocity,
            MovementDirection::Backward => self.camera.eye += self.camera.center * velocity,
            MovementDirection::Up => self.camera.eye += self.world_up * velocity,
            MovementDirection::Down => self.camera.eye -= self.world_up * velocity,
            MovementDirection::Left => self.camera.eye -= self.right * velocity,
            MovementDirection::Right => self.camera.eye += self.right * velocity,
        }
        self.update();
    }
}
