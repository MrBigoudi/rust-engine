use engine::{
    core::debug::errors::EngineError, renderer::{renderer_frontend::renderer_get_main_camera, scene::camera::Camera}
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
    pub acceleration: f32,
    pub speed: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub is_accelerating: bool,
}

impl Default for CameraMovement {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            acceleration: 5.0,
            speed: 20.0,
            yaw_deg: 0.0,
            pitch_deg: 0.0,
            is_accelerating: false,
        }
    }
}

impl CameraMovement {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            camera: renderer_get_main_camera()?,
            ..Default::default()
        })
    }

    fn update_view(&mut self) {
        // to create a correct model view, we need to move the world in opposite direction to the camera
        // so we will create the camera model matrix and invert
        let translation = glam::Mat4::from_translation(self.camera.eye);
        let rotation = self.get_rotation_matrix();
        self.camera.view = translation.mul_mat4(&rotation).inverse();
    }

    fn get_rotation_matrix(&self) -> glam::Mat4 {
        // fairly typical FPS style camera. we join the pitch and yaw rotations into the final rotation matrix
        let pitch_rotation = glam::Quat::from_axis_angle(glam::Vec3::new(1.0, 0.0, 0.0), self.pitch_deg.to_radians());
        let yaw_rotation = glam::Quat::from_axis_angle(glam::Vec3::new(0.0, -1.0, 0.0), self.yaw_deg.to_radians());
        glam::Mat4::from_quat(yaw_rotation).mul_mat4(&glam::Mat4::from_quat(pitch_rotation))
    }

    fn update(&mut self, velocity: glam::Vec3) {
        // calculate the new center vector
        let camera_rotation = self.get_rotation_matrix();
        let new_position = camera_rotation.mul_vec4(glam::Vec4::new(velocity.x, velocity.y, velocity.z, 1.0));
        self.camera.eye += glam::Vec3::new(new_position.x, new_position.y, new_position.z);
        self.update_view();
    }

    pub fn handle_movement(&mut self, direction: MovementDirection, delta_time: f64) {
        let mut velocity = self.speed * delta_time as f32;
        if self.is_accelerating {
            velocity *= self.acceleration;
        }
        let velocity = velocity * (
            match direction {
                MovementDirection::Forward => glam::Vec3::new(0.0, 0.0, 1.0),
                MovementDirection::Backward => glam::Vec3::new(0.0, 0.0, -1.0),
                MovementDirection::Up => glam::Vec3::new(0.0, 1.0, 0.0),
                MovementDirection::Down => glam::Vec3::new(0.0, -1.0, 0.0),
                MovementDirection::Right => glam::Vec3::new(1.0, 0.0, 0.0),
                MovementDirection::Left => glam::Vec3::new(-1.0, 0.0, 0.0),
            }
        );
        self.update(velocity);
    }
}
