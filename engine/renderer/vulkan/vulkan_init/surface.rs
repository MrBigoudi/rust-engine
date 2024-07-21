use ash::{khr::surface, vk::SurfaceKHR};

use crate::{
    core::debug::errors::EngineError, error, platforms::platform::Platform,
    renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn get_surface_loader(&self) -> Result<&surface::Instance, EngineError> {
        match &self.context.surface_loader {
            Some(surface) => Ok(surface),
            None => {
                error!("Can't access the vulkan surface loader");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn get_surface(&self) -> Result<&SurfaceKHR, EngineError> {
        match &self.context.surface {
            Some(surface) => Ok(surface),
            None => {
                error!("Can't access the vulkan surface");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn surface_init(&mut self, platform: &dyn Platform) -> Result<(), EngineError> {
        // init the loader
        let surface_loader = surface::Instance::new(self.get_entry()?, self.get_instance()?);

        // init the platform specific surface
        let surface = match platform.get_vulkan_surface(&self.context) {
            Ok(surface) => surface,
            Err(err) => {
                error!("Failed to create the vulkan surface: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        };

        self.context.surface_loader = Some(surface_loader);
        self.context.surface = Some(surface);

        Ok(())
    }

    pub fn surface_shutdown(&mut self) -> Result<(), EngineError> {
        unsafe {
            self.get_surface_loader()?
                .destroy_surface(*self.get_surface()?, self.get_allocator()?);
        }
        Ok(())
    }
}
