use crate::{core::debug::errors::EngineError, debug, error, platforms::platform::Platform};

use super::vulkan_types::VulkanRendererBackend;

pub mod allocator;
pub mod debug;
pub mod devices;
pub mod entry;
pub mod instance;
pub mod surface;
pub mod swapchain;

impl VulkanRendererBackend<'_> {
    pub fn vulkan_init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        if let Err(err) = self.entry_init() {
            error!("Failed to initialize the vulkan entry: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan entry initialized successfully !");
        }

        if let Err(err) = self.allocator_init() {
            error!("Failed to initialize the vulkan allocator: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan allocator initialized successfully !");
        }

        if let Err(err) = self.instance_init(application_name, platform) {
            error!("Failed to initialize the vulkan instance: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan instance initialized successfully !");
        }

        #[cfg(debug_assertions)]
        {
            if let Err(err) = self.debugger_init() {
                error!("Failed to initialize the vulkan debugger: {:?}", err);
                return Err(EngineError::InitializationFailed);
            } else {
                debug!("Vulkan debugger initialized successfully !");
            }
        }

        if let Err(err) = self.surface_init(platform) {
            error!("Failed to initialize the vulkan surface: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan surface initialized successfully !");
        }

        if let Err(err) = self.device_requirements_init() {
            error!(
                "Failed to initialize the vulkan device requirements: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan device requirements initialized successfully !");
        }

        if let Err(err) = self.physical_device_init() {
            error!("Failed to initialize the vulkan physical device: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan physical device initialized successfully !");
        }

        if let Err(err) = self.device_init() {
            error!("Failed to initialize the vulkan logical device: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan logical device initialized successfully !");
        }

        if let Err(err) = self.queues_init() {
            error!(
                "Failed to initialize the vulkan logical device queues: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan logical device queues initialized successfully !");
        }

        if let Err(err) = self.swapchain_init() {
            error!("Failed to initialize the vulkan swapchain: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan swapchain initialized successfully !");
        }

        Ok(())
    }

    pub fn vulkan_shutdown(&mut self) -> Result<(), EngineError> {
        if let Err(err) = self.swapchain_shutdown() {
            error!("Failed to shutdown the vulkan swapchain: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan swapchain shutdowned successfully !");
        }

        if let Err(err) = self.queues_shutdown() {
            error!(
                "Failed to shutdown the vulkan logical device queues: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan logical device queues shutdowned successfully !");
        }

        if let Err(err) = self.device_shutdown() {
            error!("Failed to shutdown the vulkan logical device: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan logical device shutdowned successfully !");
        }

        if let Err(err) = self.physical_device_shutdown() {
            error!("Failed to shutdown the vulkan physical device: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan physical device shutdowned successfully !");
        }

        if let Err(err) = self.device_requirements_shutdown() {
            error!(
                "Failed to shutdown the vulkan device requirements: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan device requirements shutdowned successfully !");
        }

        if let Err(err) = self.surface_shutdown() {
            error!("Failed to shutdown the vulkan surface: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan surface shutdowned successfully !");
        }

        #[cfg(debug_assertions)]
        {
            if let Err(err) = self.debugger_shutdown() {
                error!("Failed to shutdown the vulkan debugger: {:?}", err);
                return Err(EngineError::ShutdownFailed);
            } else {
                debug!("Vulkan debugger shutdowned successfully !");
            }
        }

        if let Err(err) = self.instance_shutdown() {
            error!("Failed to shutdown the vulkan instance: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan instance shutdowned successfully !");
        }

        if let Err(err) = self.allocator_shutdown() {
            error!("Failed to shutdown the vulkan allocator: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan allocator shutdowned successfully !");
        }

        if let Err(err) = self.entry_shutdown() {
            error!("Failed to shutdown the vulkan entry: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan entry shutdowned successfully !");
        }

        Ok(())
    }
}
