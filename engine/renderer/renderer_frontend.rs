use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use image::ImageReader;
use once_cell::sync::Lazy;

use crate::{
    core::debug::errors::EngineError,
    error,
    platforms::platform::Platform,
    renderer::renderer_types::GeometryRenderData,
    resources::texture::{Texture, TextureCreatorParameters},
    warn,
};

use super::{
    renderer_backend::{renderer_backend_init, RendererBackend},
    renderer_types::{RenderFrameData, RendererBackendType},
    scene::camera::{Camera, CameraCreatorParameters},
};

#[derive(Default)]
pub(crate) struct RendererFrontend {
    pub backend: Option<Box<dyn RendererBackend>>,
    pub main_camera: Option<Camera>,

    // TODO: temporary
    pub default_texture: Option<Box<dyn Texture>>,
}

impl RendererFrontend {
    pub fn set_main_camera(&mut self, new_camera: &Camera) {
        let camera: &mut Camera = self.main_camera.as_mut().unwrap();
        camera.set_view(new_camera.view);
    }

    fn init_default_texture(&mut self) -> Result<(), EngineError> {
        // NOTE: Create default texture, a 256x256 blue/white checkerboard pattern
        // This is done in code to eliminate asset dependencies
        let tex_dimension = 256u32;
        let nb_channels = 4u8;
        let pixel_count = (tex_dimension * tex_dimension) as usize;
        let mut pixels = vec![255u8; pixel_count * nb_channels as usize];
        for row in 0..tex_dimension {
            for col in 0..tex_dimension {
                let index: usize = (row * tex_dimension + col) as usize * nb_channels as usize;
                if row % 2 != 0 || col % 2 == 0 {
                    pixels[index] = 0;
                    pixels[index + 1] = 0;
                }
            }
        }
        let texture_params = TextureCreatorParameters {
            name: "default texture",
            auto_release: false,
            width: tex_dimension,
            height: tex_dimension,
            nb_channels,
            pixels: &pixels,
            has_transparency: false,
            is_default: true,
        };
        let texture = match self.create_texture(texture_params) {
            Ok(texture) => texture,
            Err(err) => {
                error!("Failed to create the default texture: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        };
        self.default_texture = Some(texture);
        Ok(())
    }

    fn init_renderer_backend(
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

    fn init_default_camera(&mut self) -> Result<(), EngineError> {
        self.main_camera = Some(Camera::new(
            CameraCreatorParameters::default(),
            self.backend.as_ref().unwrap().get_aspect_ratio()?,
        ));
        Ok(())
    }

    pub(crate) fn init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        self.init_renderer_backend(application_name, platform)?;
        // Default camera
        self.init_default_camera()?;
        // Default texture
        self.init_default_texture()?;
        Ok(())
    }

    fn destroy_default_texture(&mut self) -> Result<(), EngineError> {
        match &self.default_texture {
            Some(texture) => {
                if let Err(err) = self
                    .backend
                    .as_ref()
                    .unwrap()
                    .destroy_texture(texture.as_ref())
                {
                    error!("Failed to destroy the default texture: {:?}", err);
                    return Err(EngineError::ShutdownFailed);
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn destroy_default_camera(&mut self) -> Result<(), EngineError> {
        // if needed
        Ok(())
    }

    fn destroy_renderer_backend(&mut self) -> Result<(), EngineError> {
        if let Err(err) = self.backend.as_mut().unwrap().shutdown() {
            error!("Failed to shutdown the renderer backend: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
        Ok(())
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), EngineError> {
        self.destroy_default_texture()?;
        self.destroy_default_camera()?;
        self.destroy_renderer_backend()?;
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
                    if let Err(err) = self.backend.as_mut().unwrap().update_global_state(
                        camera.projection,
                        camera.view,
                        glam::Vec3::ZERO,
                        glam::Vec4::ONE,
                        0,
                    ) {
                        error!(
                            "Failed to update the renderer backend global state: {:?}",
                            err
                        );
                        return Err(EngineError::Unknown);
                    }

                    // mat4 model = mat4_translation((vec3){0, 0, 0});
                    // static mut ANGLE: f32 = 0.01;
                    // unsafe { ANGLE += 0.001 };
                    // let rotation =
                    //     glam::Quat::from_axis_angle(glam::Vec3::new(0.0, 0.0, -1.0), unsafe {
                    //         ANGLE
                    //     });
                    // let model = glam::Mat4::from_quat(rotation);
                    let default_texture = self
                        .default_texture
                        .as_ref()
                        .map(|texture| texture.clone_box());
                    let geometry_data = GeometryRenderData::default()
                        .model(glam::Mat4::IDENTITY)
                        .texture(0, default_texture)
                        .object_id(Some(0)) // TODO: actual object id
                    ;
                    if let Err(err) = self.backend.as_mut().unwrap().update_object(&geometry_data) {
                        error!("Failed to update the renderer backend objects: {:?}", err);
                        return Err(EngineError::Unknown);
                    }
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

    pub fn create_texture(
        &self,
        params: TextureCreatorParameters,
    ) -> Result<Box<dyn Texture>, EngineError> {
        self.backend.as_ref().unwrap().create_texture(params)
    }

    pub fn load_texture(&self, path: &Path, name: &str) -> Result<Box<dyn Texture>, EngineError> {
        // TODO: Better path handling
        let image = match ImageReader::open(path) {
            Ok(image) => image,
            Err(err) => {
                error!(
                    "Failed to open the file: {:?}, when trying to load a texture: {:?}",
                    path, err
                );
                return Err(EngineError::IO);
            }
        };
        let image = match image.decode() {
            Ok(image) => image,
            Err(err) => {
                error!(
                    "Failed to decode the file: {:?}, when trying to load a texture: {:?}",
                    path, err
                );
                return Err(EngineError::IO);
            }
        };
        // TODO: handle different formats
        let image = image.to_rgba8();
        let mut has_transparency = false;
        for pixel in image.pixels() {
            if pixel[3] < 255 {
                has_transparency = true; // Transparency found
            }
        }

        let texture_parameters = TextureCreatorParameters {
            name,
            auto_release: true,
            width: image.width(),
            height: image.height(),
            nb_channels: 4, // for now
            pixels: image.as_raw(),
            has_transparency,
            is_default: self.default_texture.is_some()
                && self
                    .default_texture
                    .as_ref()
                    .unwrap()
                    .get_generation()
                    .is_some(),
        };

        // Acquire internal texture resources and upload to GPU
        let new_texture = match self.create_texture(texture_parameters) {
            Ok(texture) => texture,
            Err(err) => {
                error!(
                    "Failed to create the backend texture when creating a frontend texture: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        Ok(new_texture)
    }

    fn update_default_texture(&mut self, new_texture: Box<dyn Texture>) -> Result<(), EngineError> {
        // Destroy Old texture
        if let Some(texture) = &self.default_texture {
            if let Err(err) = self
                .backend
                .as_ref()
                .unwrap()
                .destroy_texture(texture.as_ref())
            {
                error!("Failed to destroy the old texture when updating the renderer default's texture: {:?}", err);
                return Err(EngineError::ShutdownFailed);
            }
        }
        self.default_texture = Some(new_texture);
        Ok(())
    }

    // TODO: temporary test code
    pub fn swap_default_texture(&mut self) -> Result<(), EngineError> {
        // Get the current working directory
        let crate_dir = env!("CARGO_MANIFEST_DIR");
        let paths: [PathBuf; 2] = [
            Path::new(crate_dir).join("assets/textures/cobblestone.png"),
            Path::new(crate_dir).join("assets/textures/paving.png"),
        ];
        let names = ["cobblestone", "paving"];

        static mut CUR_CHOICE: usize = 0;
        unsafe { CUR_CHOICE = (CUR_CHOICE + 1) % names.len() };

        let new_texture =
            match self.load_texture(&paths[unsafe { CUR_CHOICE }], names[unsafe { CUR_CHOICE }]) {
                Ok(texture) => texture,
                Err(err) => {
                    error!(
                        "Failed to load a new texture when swapping the default texture: {:?}",
                        err
                    );
                    return Err(EngineError::InitializationFailed);
                }
            };

        if let Err(err) = self.update_default_texture(new_texture) {
            error!(
                "Failed to update the default texture when swapping the default texture: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        };

        Ok(())
    }
    // TODO: end of temporary code
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

pub fn renderer_get_default_texture() -> Result<&'static dyn Texture, EngineError> {
    let front_end = fetch_global_renderer(EngineError::UpdateFailed)?;
    Ok(front_end.default_texture.as_ref().unwrap().as_ref())
}

// TODO: temporary code
pub fn renderer_swap_default_texture() -> Result<(), EngineError> {
    let front_end = fetch_global_renderer(EngineError::UpdateFailed)?;
    front_end.swap_default_texture()
}
// TODO: end of temporary code
