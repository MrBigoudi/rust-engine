use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{core::debug::errors::EngineError, error, platforms::platform::Platform, warn};

use super::{
    renderer_backend::{renderer_backend_init, RendererBackend},
    renderer_types::{RenderFrameData, RendererBackendType},
};

#[derive(Default)]
pub(crate) struct RendererFrontend {
    pub backend: Option<Box<dyn RendererBackend>>,
}

impl RendererFrontend {
    pub fn init(
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
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), EngineError> {
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

    pub fn draw_frame(&mut self, frame_data: &RenderFrameData) -> Result<(), EngineError> {
        // If the begin frame returned successfully, mid-frame operations may continue.
        match self.begin_frame(frame_data.delta_time) {
            Err(err) => {
                error!("Failed to begin the renderer frontend frame: {:?}", err);
                Err(EngineError::Unknown)
            }
            Ok(true) => {
                // TODO: temporary test code
                {
                    let projection = glam::Mat4::perspective_lh(
                        (45f32).to_radians(),
                        self.backend.as_ref().unwrap().get_aspect_ratio()?,
                        0.1,
                        1000.0,
                    );
                    static mut Z: f32 = -1.0;
                    unsafe { Z -= 0.005 };
                    let view = glam::Mat4::look_at_lh(
                        glam::Vec3::new(0.0, 0.0, unsafe { Z }),
                        glam::Vec3::ZERO,
                        glam::Vec3::new(0.0, 1.0, 0.0),
                    );
                    // crate::debug!("\n\tproj: {:?}\n\tview: {:?}\n\n", projection.to_string(), view.to_string());
                    self.backend.as_mut().unwrap().update_global_state(
                        projection,
                        view,
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

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        if let Err(err) = self.backend.as_mut().unwrap().resize(width, height) {
            error!("Failed to resize the renderer frontend: {:?}", err);
            return Err(EngineError::Unknown);
        }
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
