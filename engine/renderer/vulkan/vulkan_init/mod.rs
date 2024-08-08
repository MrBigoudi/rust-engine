use crate::{core::debug::errors::EngineError, debug, error, platforms::platform::Platform};

use super::{vulkan_types::VulkanRendererBackend, vulkan_utils::buffer::BufferCommandParameters};

pub mod allocator;
pub mod command_buffer;
pub mod command_pool;
pub mod debug;
pub mod devices;
pub mod entry;
pub mod framebuffer;
pub mod instance;
pub mod objects;
pub mod renderpass;
pub mod shaders;
pub mod surface;
pub mod swapchain;
pub mod sync_structures;

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

        if let Err(err) = self.framebuffer_dimensions_init() {
            error!(
                "Failed to initialize the vulkan framebuffer dimensions: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan framebuffer dimensions initialized successfully: (width={:?}, height={:?})!",
            self.framebuffer_width, self.framebuffer_height
            );
        }

        if let Err(err) = self.swapchain_init() {
            error!("Failed to initialize the vulkan swapchain: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan swapchain initialized successfully !");
        }

        if let Err(err) = self.renderpass_init() {
            error!("Failed to initialize the vulkan renderpass: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan renderpass initialized successfully !");
        }

        if let Err(err) = self.graphics_command_pool_init() {
            error!(
                "Failed to initialize the vulkan graphics command pool: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan graphics command pool initialized successfully !");
        }

        if let Err(err) = self.graphics_command_buffers_init() {
            error!(
                "Failed to initialize the vulkan graphics command buffers: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan graphics command buffers initialized successfully !");
        }

        if let Err(err) = self.swapchain_framebuffers_init() {
            error!(
                "Failed to initialize the vulkan swapchain framebuffers: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan swapchain framebuffers initialized successfully !");
        }

        if let Err(err) = self.sync_structures_init() {
            error!("Failed to initialize the vulkan sync structures: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan sync structures initialized successfully !");
        }

        if let Err(err) = self.builtin_shaders_init() {
            error!("Failed to initialize the vulkan builtin shaders: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan builtin shaders initialized successfully !");
        }

        if let Err(err) = self.objects_buffers_init() {
            error!("Failed to initialize the vulkan objects buffers: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan objects buffers initialized successfully !");
        }

        // TODO: temporary test code
        {
            let mut vertices = vec![
                glam::Vec3::new(0.0, -0.5, 0.0),
                glam::Vec3::new(0.5, 0.5, 0.0),
                glam::Vec3::new(0.0, 0.5, 0.0),
                glam::Vec3::new(0.5, -0.5, 0.0),
            ];
            let mut indices: Vec<u32> = vec![0, 1, 2, 0, 3, 1];
            let vertices_command_parameters = BufferCommandParameters {
                command_pool: self.get_graphics_command_pool()?,
                fence: &ash::vk::Fence::null(),
                queue: self.get_queues()?.graphics_queue.unwrap(),
            };
            let vertices_buffer = &self.get_objects_buffers()?.vertex_buffer;
            let vertices_size = size_of::<glam::Vec3>() * vertices.len();
            self.upload_data_range(
                vertices_command_parameters,
                &vertices_buffer,
                0,
                vertices_size,
                vertices.as_mut_ptr() as *mut std::ffi::c_void,
            )?;

            let indices_command_parameters = BufferCommandParameters {
                command_pool: self.get_graphics_command_pool()?,
                fence: &ash::vk::Fence::null(),
                queue: self.get_queues()?.graphics_queue.unwrap(),
            };
            let indices_buffer = &self.get_objects_buffers()?.index_buffer;
            let indices_size = size_of::<u32>() * indices.len();
            self.upload_data_range(
                indices_command_parameters,
                &indices_buffer,
                0,
                indices_size,
                indices.as_mut_ptr() as *mut std::ffi::c_void,
            )?;
        }
        // TODO: end temp code

        Ok(())
    }

    pub fn vulkan_shutdown(&mut self) -> Result<(), EngineError> {
        self.device_wait_idle()?;

        if let Err(err) = self.objects_buffers_shutdown() {
            error!("Failed to shutdown the vulkan objects buffers: {:?}", err);
            return Err(EngineError::InitializationFailed);
        } else {
            debug!("Vulkan objects buffers shutted down successfully !");
        }

        if let Err(err) = self.builtin_shaders_shutdown() {
            error!("Failed to shutdown the vulkan builtin shaders: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan builtin shaders shutted down successfully !");
        }

        if let Err(err) = self.sync_structures_shutdown() {
            error!("Failed to shutdown the vulkan sync structures: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan sync structures shutted down successfully !");
        }

        if let Err(err) = self.swapchain_framebuffers_shutdown() {
            error!(
                "Failed to shutdown the vulkan swapchain framebuffers: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan swapchain framebuffers shutted down successfully !");
        }

        if let Err(err) = self.graphics_command_buffers_shutdown() {
            error!(
                "Failed to shutdown the vulkan graphics command buffers: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan graphics command buffers shutted down successfully !");
        }

        if let Err(err) = self.graphics_command_pool_shutdown() {
            error!(
                "Failed to shutdown the vulkan graphics command pool: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan graphics command pool shutted down successfully !");
        }

        if let Err(err) = self.renderpass_shutdown() {
            error!("Failed to shutdown the vulkan renderpass: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan renderpass shutted down successfully !");
        }

        if let Err(err) = self.swapchain_shutdown() {
            error!("Failed to shutdown the vulkan swapchain: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan swapchain shutted down successfully !");
        }

        if let Err(err) = self.queues_shutdown() {
            error!(
                "Failed to shutdown the vulkan logical device queues: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan logical device queues shutted down successfully !");
        }

        if let Err(err) = self.device_shutdown() {
            error!("Failed to shutdown the vulkan logical device: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan logical device shutted down successfully !");
        }

        if let Err(err) = self.physical_device_shutdown() {
            error!("Failed to shutdown the vulkan physical device: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan physical device shutted down successfully !");
        }

        if let Err(err) = self.device_requirements_shutdown() {
            error!(
                "Failed to shutdown the vulkan device requirements: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan device requirements shutted down successfully !");
        }

        if let Err(err) = self.surface_shutdown() {
            error!("Failed to shutdown the vulkan surface: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan surface shutted down successfully !");
        }

        #[cfg(debug_assertions)]
        {
            if let Err(err) = self.debugger_shutdown() {
                error!("Failed to shutdown the vulkan debugger: {:?}", err);
                return Err(EngineError::ShutdownFailed);
            } else {
                debug!("Vulkan debugger shutted down successfully !");
            }
        }

        if let Err(err) = self.instance_shutdown() {
            error!("Failed to shutdown the vulkan instance: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan instance shutted down successfully !");
        }

        if let Err(err) = self.allocator_shutdown() {
            error!("Failed to shutdown the vulkan allocator: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan allocator shutted down successfully !");
        }

        if let Err(err) = self.entry_shutdown() {
            error!("Failed to shutdown the vulkan entry: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        } else {
            debug!("Vulkan entry shutted down successfully !");
        }

        Ok(())
    }
}
