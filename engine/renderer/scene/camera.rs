#[derive(Clone, Copy, Debug)]
pub enum ProjectionType {
    Orthographic,
    Perspective,
}

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub view: glam::Mat4,
    pub projection_type: ProjectionType,
    pub projection: glam::Mat4,
    pub near_clip: f32,
    pub far_clip: f32,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub eye: glam::Vec3,
    pub center: glam::Vec3,
    pub up: glam::Vec3,
}

pub struct CameraCreatorParameters {
    pub near_clip: f32,
    pub far_clip: f32,
    pub fov: f32,
    pub eye: glam::Vec3,
    pub center: glam::Vec3,
    pub up: glam::Vec3,
    pub projection: ProjectionType,
}

impl Default for CameraCreatorParameters {
    fn default() -> Self {
        Self {
            near_clip: 0.1,
            far_clip: 1000.0,
            fov: (45f32).to_radians(),
            eye: glam::Vec3::new(0.0, 0.0, -1.0),
            center: glam::Vec3::ZERO,
            up: glam::Vec3::new(0.0, 1.0, 0.0),
            projection: ProjectionType::Perspective,
        }
    }
}

impl CameraCreatorParameters {
    pub fn near_clip(mut self, near_clip: f32) -> Self {
        self.near_clip = near_clip;
        self
    }

    pub fn far_clip(mut self, far_clip: f32) -> Self {
        self.far_clip = far_clip;
        self
    }

    pub fn fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    pub fn eye(mut self, eye: glam::Vec3) -> Self {
        self.eye = eye;
        self
    }

    pub fn center(mut self, center: glam::Vec3) -> Self {
        self.center = center;
        self
    }

    pub fn up(mut self, up: glam::Vec3) -> Self {
        self.up = up;
        self
    }

    pub fn projection(mut self, projection: ProjectionType) -> Self {
        self.projection = projection;
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new(CameraCreatorParameters::default(), 16.0 / 9.0)
    }
}

impl Camera {
    pub fn new(parameters: CameraCreatorParameters, aspect_ratio: f32) -> Self {
        let view = glam::Mat4::look_at_lh(parameters.eye, parameters.center, parameters.up);
        let projection = match parameters.projection {
            ProjectionType::Orthographic => todo!("Orthographic not implemented"),
            ProjectionType::Perspective => glam::Mat4::perspective_lh(
                parameters.fov,
                aspect_ratio,
                parameters.near_clip,
                parameters.far_clip,
            ),
        };
        Self {
            view,
            projection_type: parameters.projection,
            projection,
            near_clip: parameters.near_clip,
            far_clip: parameters.far_clip,
            fov: parameters.fov,
            aspect_ratio,
            eye: parameters.eye,
            center: parameters.center,
            up: parameters.up,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        let projection = match self.projection_type {
            ProjectionType::Orthographic => todo!("Orthographic not implemented"),
            ProjectionType::Perspective => {
                glam::Mat4::perspective_lh(self.fov, aspect_ratio, self.near_clip, self.far_clip)
            }
        };
        self.projection = projection;
        self.aspect_ratio = aspect_ratio;
    }

    pub fn set_view(&mut self, view: glam::Mat4) {
        self.view = view;
    }
}
