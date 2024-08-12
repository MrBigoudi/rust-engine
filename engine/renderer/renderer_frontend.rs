use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{core::debug::errors::EngineError, error, platforms::platform::Platform, warn};

use super::{
    renderer_backend::{renderer_backend_init, RendererBackend},
    renderer_types::{RenderFrameData, RendererBackendType},
    scene::camera::{Camera, CameraCreatorParameters},
};

#[derive(Default)]
pub(crate) struct RendererFrontend {
    pub backend: Option<Box<dyn RendererBackend>>,
    pub main_camera: Option<Camera>,
}

impl RendererFrontend {
    pub fn set_main_camera(&mut self, new_camera: &Camera) {
        let camera: &mut Camera = self.main_camera.as_mut().unwrap();
        camera.clone_from(new_camera);
    }

    pub(crate) fn init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        // TODO: make this configurable
        let backend =
            match renderer_backend_init(RendererBackendType::Vulkan, application_name, platform) {
                Ok(backend) => backend,
                Err(err) => {
                    error!("Failed to initialize the renderer backend: {:?}", err);
                    return Err(EngineError::InitializationFailed);
                }
            };
        self.backend = Some(Box::new(backend));
        // Default camera
        self.main_camera = Some(Camera::new(
            CameraCreatorParameters::default(),
            self.backend.as_ref().unwrap().get_aspect_ratio()?,
        ));
        Ok(())
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), EngineError> {
        match self.backend.as_mut().unwrap().shutdown() {
            Ok(()) => (),
            Err(err) => {
                error!("Failed to shutdown the renderer backend: {:?}", err);
                return Err(EngineError::ShutdownFailed);
            }
        }
        Ok(())
    }

    fn begin_frame(&mut self, delta_time: f64) -> Result<bool, EngineError> {
        match self.backend.as_mut().unwrap().begin_frame(delta_time) {
            Ok(val) => Ok(val),
            Err(err) => {
                error!("Failed to begin the renderer backend frame: {:?}", err);
                Err(EngineError::Unknown)
            }
        }
    }

    fn end_frame(&mut self, delta_time: f64) -> Result<(), EngineError> {
        match self.backend.as_mut().unwrap().end_frame(delta_time) {
            Ok(()) => (),
            Err(err) => {
                error!("Failed to end the renderer backend frame: {:?}", err);
                return Err(EngineError::Unknown);
            }
        };
        match self.backend.as_mut().unwrap().increase_frame_number() {
            Ok(()) => (),
            Err(err) => {
                error!(
                    "Failed to increase the number of frames in the renderer backend: {:?}",
                    err
                );
                return Err(EngineError::Unknown);
            }
        };
        Ok(())
    }

    pub(crate) fn draw_frame(&mut self, frame_data: &RenderFrameData) -> Result<(), EngineError> {
        // If the begin frame returned successfully, mid-frame operations may continue.
        match self.begin_frame(frame_data.delta_time) {
            Err(err) => {
                error!("Failed to begin the renderer frontend frame: {:?}", err);
                Err(EngineError::Unknown)
            }
            Ok(true) => {
                // TODO: temporary test code
                {
                    let camera = self.main_camera.unwrap();
                    self.backend.as_mut().unwrap().update_global_state(
                        camera.projection,
                        camera.view,
                        glam::Vec3::ZERO,
                        glam::Vec4::ONE,
                        0,
                    )?;

                    // mat4 model = mat4_translation((vec3){0, 0, 0});
                    static mut ANGLE: f32 = 0.01;
                    unsafe { ANGLE += 0.001 };
                    let rotation =
                        glam::Quat::from_axis_angle(glam::Vec3::new(0.0, 0.0, -1.0), unsafe {
                            ANGLE
                        });
                    let model = glam::Mat4::from_quat(rotation);
                    self.backend.as_mut().unwrap().update_object(model)?;
                }
                // TODO: temporary test code

                // End the frame. If this fails, it is likely unrecoverable
                match self.end_frame(frame_data.delta_time) {
                    Err(err) => {
                        error!("Failed to end the renderer frontend frame: {:?}", err);
                        Err(EngineError::Unknown)
                    }
                    Ok(()) => Ok(()),
                }
            }
            Ok(false) => {
                warn!("Could not begin the frame, skipping it");
                Ok(())
            }
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        if let Err(err) = self.backend.as_mut().unwrap().resize(width, height) {
            error!("Failed to resize the renderer frontend: {:?}", err);
            return Err(EngineError::Unknown);
        }
        let new_aspect_ratio = self.backend.as_ref().unwrap().get_aspect_ratio()?;
        let camera: &mut Camera = match self.main_camera.as_mut() {
            None => return Ok(()),
            Some(camera) => camera,
        };
        camera.update_aspect_ratio(new_aspect_ratio);
        Ok(())
    }
}

pub(crate) static mut GLOBAL_RENDERER: Lazy<Mutex<RendererFrontend>> = Lazy::new(Mutex::default);

pub(crate) fn fetch_global_renderer(
    error: EngineError,
) -> Result<&'static mut RendererFrontend, EngineError> {
    unsafe {
        match GLOBAL_RENDERER.get_mut() {
            Ok(renderer) => Ok(renderer),
            Err(err) => {
                error!("Failed to fetch the global renderer: {:?}", err);
                Err(error)
            }
        }
    }
}

/// Initiate the engine renderer
pub(crate) fn renderer_init(
    application_name: &str,
    platform: &dyn Platform,
) -> Result<(), EngineError> {
    let global_renderer = fetch_global_renderer(EngineError::InitializationFailed)?;
    match global_renderer.init(application_name, platform) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to initialize the renderer: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
    }
    Ok(())
}

pub(crate) fn renderer_draw_frame(frame_data: &RenderFrameData) -> Result<(), EngineError> {
    let global_renderer = fetch_global_renderer(EngineError::InitializationFailed)?;
    match global_renderer.draw_frame(frame_data) {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to render a frame: {:?}", err);
            return Err(EngineError::Unknown);
        }
    }
    Ok(())
}

/// Shutdown the engine renderer
pub(crate) fn renderer_shutdown() -> Result<(), EngineError> {
    let global_renderer = fetch_global_renderer(EngineError::InitializationFailed)?;
    match global_renderer.shutdown() {
        Ok(()) => (),
        Err(err) => {
            error!("Failed to shutdown the renderer: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
    }
    unsafe {
        // Empty GLOBAL_EVENTS
        GLOBAL_RENDERER = Lazy::new(Mutex::default);
    }
    Ok(())
}

// TODO: put it back to crate visibility
pub fn renderer_set_main_camera(new_camera: &Camera) -> Result<(), EngineError> {
    let front_end = fetch_global_renderer(EngineError::UpdateFailed)?;
    front_end.set_main_camera(new_camera);
    Ok(())
}

pub fn renderer_get_main_camera() -> Result<Camera, EngineError> {
    let front_end = fetch_global_renderer(EngineError::UpdateFailed)?;
    Ok(front_end.main_camera.unwrap())
}
