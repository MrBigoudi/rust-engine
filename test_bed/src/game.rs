use engine::{
    game::Game,
    renderer::{
        renderer_frontend::{renderer_get_main_camera, renderer_set_main_camera},
        scene::camera::Camera,
    },
};

#[derive(Default)]
pub struct TestBedGame {
    pub camera: Option<Camera>,
}

impl Game for TestBedGame {
    fn on_start(&mut self) -> Result<(), engine::core::debug::errors::EngineError> {
        self.camera = Some(renderer_get_main_camera()?);
        Ok(())
    }

    fn on_update(
        &mut self,
        _delta_time: f64,
    ) -> Result<(), engine::core::debug::errors::EngineError> {
        static mut Z: f32 = -1.0;
        unsafe { Z -= 0.005 };
        let new_eye = glam::Vec3::new(0.0, 0.0, unsafe { Z });
        let new_center = glam::Vec3::ZERO;
        let new_up = glam::Vec3::new(0.0, 1.0, 0.0);
        let camera: &mut Camera = self.camera.as_mut().unwrap();
        camera.set_view(new_eye, new_center, new_up);
        renderer_set_main_camera(camera)?;

        Ok(())
    }
}
