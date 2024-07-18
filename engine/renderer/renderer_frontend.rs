use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::{core::errors::EngineError, error, platforms::platform::Platform};

use super::{
    renderer_backend::{renderer_backend_init, RendererBackend},
    renderer_types::{RenderFrame, RendererBackendType},
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

    fn begin_frame(&mut self, delta_time: f64) -> Result<(), EngineError> {
        match self.backend.as_mut().unwrap().begin_frame(delta_time) {
            Ok(()) => (),
            Err(err) => {
                error!("Failed to begin the renderer backend frame: {:?}", err);
                return Err(EngineError::Unknown);
            }
        }
        Ok(())
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

    pub fn draw_frame(&mut self, frame_data: &RenderFrame) -> Result<(), EngineError> {
        // If the begin frame returned successfully, mid-frame operations may continue.
        match self.begin_frame(frame_data.delta_time) {
            Err(err) => {
                error!("Failed to begin the renderer frontend frame: {:?}", err);
                Err(EngineError::Unknown)
            }
            Ok(()) => {
                // End the frame. If this fails, it is likely unrecoverable.
                match self.end_frame(frame_data.delta_time) {
                    Err(err) => {
                        error!("Failed to end the renderer frontend frame: {:?}", err);
                        Err(EngineError::Unknown)
                    },
                    Ok(()) => Ok(()),
                }
            }
        }
    }
}

pub(crate) static mut GLOBAL_RENDERER: Lazy<Mutex<RendererFrontend>> = Lazy::new(Mutex::default);

fn fetch_global_renderer(error: EngineError) -> Result<&'static mut RendererFrontend, EngineError> {
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
        },
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
        },
    }
    unsafe {
        // Empty GLOBAL_EVENTS
        GLOBAL_RENDERER = Lazy::new(Mutex::default);
    }
    Ok(())
}
