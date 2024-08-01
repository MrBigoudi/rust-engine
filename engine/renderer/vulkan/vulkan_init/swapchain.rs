use std::cmp::{max, min};

use ash::{
    khr::swapchain,
    vk::{
        ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Fence, Format, Image, ImageAspectFlags,
        ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, ImageViewCreateInfo,
        ImageViewType, MemoryPropertyFlags, PhysicalDevice, PresentInfoKHR, PresentModeKHR,
        Semaphore, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SwapchainCreateInfoKHR,
        SwapchainKHR,
    },
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{
        vulkan_types::VulkanRendererBackend,
        vulkan_utils::{self, image::ImageCreatorParameters},
    },
    warn,
};

#[derive(Default, Debug)]
pub(crate) struct SwapchainSupportDetails {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn is_complete(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

pub(crate) struct Swapchain {
    pub device: swapchain::Device,
    pub handler: SwapchainKHR,
    pub surface_format: SurfaceFormatKHR,
    pub max_frames_in_flight: u16,
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
    pub depth_attachement: Option<vulkan_utils::image::Image>,
}

impl VulkanRendererBackend<'_> {
    pub(crate) fn query_swapchain_support(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<SwapchainSupportDetails, EngineError> {
        let surface_capabilities = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_capabilities(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        let surface_format = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_formats(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        let surface_present_modes = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_present_modes(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        Ok(SwapchainSupportDetails {
            capabilities: surface_capabilities,
            formats: surface_format,
            present_modes: surface_present_modes,
        })
    }

    fn swapchain_create_max_frames_in_flight(&mut self, nb_frames: u16) -> Result<(), EngineError> {
        let swapchain = self.context.swapchain.as_mut().unwrap();
        swapchain.max_frames_in_flight = nb_frames;
        Ok(())
    }

    fn swapchain_select_format(
        &mut self,
        prefered_format: Format,
        prefered_color_space: ColorSpaceKHR,
    ) -> Result<(), EngineError> {
        let supported_formats = self
            .get_physical_device_info()?
            .swapchain_support_details
            .formats
            .clone();
        let mut selected_format: Option<SurfaceFormatKHR> = None;
        'get_prefered_format_loop: for format in &supported_formats {
            if format.format == prefered_format && format.color_space == prefered_color_space {
                selected_format = Some(*format);
                break 'get_prefered_format_loop;
            }
        }
        let swapchain = self.context.swapchain.as_mut().unwrap();
        match selected_format {
            Some(format) => swapchain.surface_format = format,
            None => swapchain.surface_format = supported_formats[0],
        }
        Ok(())
    }

    fn swapchain_select_present_mode(
        &self,
        default_mode: PresentModeKHR,
        prefered_mode: PresentModeKHR,
    ) -> Result<PresentModeKHR, EngineError> {
        let supported_present_modes = self
            .get_physical_device_info()?
            .swapchain_support_details
            .present_modes
            .clone();
        for present_mode in &supported_present_modes {
            if *present_mode == prefered_mode {
                return Ok(prefered_mode);
            }
        }
        Ok(default_mode)
    }

    fn swpachain_create_extent(&self, width: u32, height: u32) -> Result<Extent2D, EngineError> {
        let supported_capabilities = self
            .get_physical_device_info()?
            .swapchain_support_details
            .capabilities;
        let mut extent = Extent2D { width, height };
        // Clamp to the value allowed by the GPU.
        let min_extent = supported_capabilities.min_image_extent;
        let max_extent = supported_capabilities.min_image_extent;
        extent.width = min(max_extent.width, max(min_extent.width, extent.width));
        extent.height = min(max_extent.height, max(min_extent.height, extent.height));
        Ok(extent)
    }

    fn swapchain_create_image_count(&self) -> Result<u32, EngineError> {
        let supported_capabilities = self
            .get_physical_device_info()?
            .swapchain_support_details
            .capabilities;
        let image_count = supported_capabilities.min_image_count + 1;
        if supported_capabilities.max_image_count > 0 {
            Ok(min(image_count, supported_capabilities.max_image_count))
        } else {
            Ok(image_count)
        }
    }

    fn swapchain_images_init(&mut self) -> Result<(), EngineError> {
        let swapchain = self.context.swapchain.as_mut().unwrap();
        swapchain.images = unsafe {
            let swapchain_device = &swapchain.device;
            match swapchain_device.get_swapchain_images(swapchain.handler) {
                Ok(images) => images,
                Err(err) => {
                    error!("Failed to get vulkan swapchain images: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        Ok(())
    }

    fn swapchain_image_views_init(&mut self) -> Result<(), EngineError> {
        let swapchain = self.context.swapchain.as_ref().unwrap();
        let mut new_image_views = Vec::new();
        for image in &swapchain.images {
            let subresource_range = ImageSubresourceRange::default()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let image_view_info = ImageViewCreateInfo::default()
                .image(*image)
                .format(swapchain.surface_format.format)
                .view_type(ImageViewType::TYPE_2D)
                .subresource_range(subresource_range);

            let new_image_view = unsafe {
                let device = self.get_device()?;
                match device.create_image_view(&image_view_info, self.get_allocator()?) {
                    Ok(image_views) => image_views,
                    Err(err) => {
                        error!(
                            "Failed to create new vulkan swapchain image views: {:?}",
                            err
                        );
                        return Err(EngineError::VulkanFailed);
                    }
                }
            };

            new_image_views.push(new_image_view);
        }

        {
            let swapchain = self.context.swapchain.as_mut().unwrap();
            swapchain.image_views = new_image_views;
        }

        Ok(())
    }

    fn swapchain_create_depth_images(&mut self, extent: Extent2D) -> Result<(), EngineError> {
        // Create depth image and its view.
        let depth_image_creation_parameters = ImageCreatorParameters::default()
            .height(extent.height)
            .width(extent.width)
            .image_format(self.get_physical_device_info()?.depth_format.unwrap())
            .image_tiling(ImageTiling::OPTIMAL)
            .image_usage_flags(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .memory_flags(MemoryPropertyFlags::DEVICE_LOCAL)
            .should_create_view(true)
            .image_view_aspect_flags(ImageAspectFlags::DEPTH);
        let depth_image = match self.create_image(depth_image_creation_parameters) {
            Ok(depth_image) => depth_image,
            Err(err) => {
                error!("Failed to create the vulkan depth image: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        };
        let swapchain = self.context.swapchain.as_mut().unwrap();
        swapchain.depth_attachement = Some(depth_image);

        Ok(())
    }

    fn swapchain_create_base(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        // for triple buffering, so at most writting to 2 frames at a time
        self.swapchain_create_max_frames_in_flight(2)?;
        // Choose a swap surface format.
        self.swapchain_select_format(Format::B8G8R8A8_UNORM, ColorSpaceKHR::SRGB_NONLINEAR)?;
        let image_format = self.get_swapchain()?.surface_format;
        // Choose a present mode
        let present_mode =
            self.swapchain_select_present_mode(PresentModeKHR::FIFO, PresentModeKHR::MAILBOX)?;
        // Requery swapchain support
        {
            let physical_device = *self.get_physical_device()?;
            let new_swapchain_support = self.query_swapchain_support(&physical_device)?;
            let physical_device_info = self.context.physical_device_info.as_mut().unwrap();
            physical_device_info.swapchain_support_details = new_swapchain_support;
        }
        // Create extent
        let extent = self.swpachain_create_extent(width, height)?;
        // Create image count
        let image_count = self.swapchain_create_image_count()?;

        // get the surface
        let surface = self.get_surface()?;
        // get the transform
        let pre_transform = self
            .get_physical_device_info()?
            .swapchain_support_details
            .capabilities
            .current_transform;

        let swapchain_create_info = SwapchainCreateInfoKHR::default()
            .surface(*surface)
            .image_extent(extent)
            .min_image_count(image_count)
            .image_format(image_format.format)
            .image_color_space(image_format.color_space)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(pre_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        // Setup the queue family indices
        let queues = self.get_queues()?;
        let graphics_queue_index = self.get_queues()?.graphics_family_index.unwrap() as u32;
        let present_queue_index = self.get_queues()?.present_family_index.unwrap() as u32;
        let queue_family_indices = [graphics_queue_index, present_queue_index];
        let swapchain_create_info = if graphics_queue_index != present_queue_index {
            swapchain_create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            swapchain_create_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        };

        let swapchain = unsafe {
            let swapchain_device = &self.get_swapchain()?.device;
            match swapchain_device.create_swapchain(&swapchain_create_info, self.get_allocator()?) {
                Ok(swapchain) => swapchain,
                Err(err) => {
                    error!("Failed to create a vulkan swapchain: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        self.context.swapchain.as_mut().unwrap().handler = swapchain;
        // Create images
        self.context.image_index = 0;
        self.swapchain_images_init()?;
        self.swapchain_image_views_init()?;
        // Depth resources
        self.device_detect_depth_format()?;
        self.swapchain_create_depth_images(extent)?;
        Ok(())
    }

    fn swapchain_destroy_base(&mut self) -> Result<(), EngineError> {
        // Destoy depth attachement
        let depth_image = &self.get_swapchain()?.depth_attachement;
        if let Some(depth_image) = depth_image {
            self.destroy_image(depth_image)?;
        }

        // Only destroy the views, not the images, since those are owned by the swapchain
        for image_view in &self.get_swapchain()?.image_views {
            let device = self.get_device()?;
            unsafe {
                device.destroy_image_view(*image_view, self.get_allocator()?);
            }
        }

        unsafe {
            let device = &self.get_swapchain()?.device;
            let swapchain = self.get_swapchain()?.handler;
            device.destroy_swapchain(swapchain, self.get_allocator()?)
        }

        Ok(())
    }

    pub fn swapchain_create(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        if let Err(err) = self.swapchain_create_base(width, height) {
            error!("Failed to create the initial swapchain: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
        Ok(())
    }

    pub fn swapchain_recreate(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        if let Err(err) = self.swapchain_destroy_base() {
            error!(
                "Failed to destroy previous swapchain when recreating a swapchain: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }
        if let Err(err) = self.swapchain_create_base(width, height) {
            error!("Failed to create a new swapchain: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }
        Ok(())
    }

    pub fn swapchain_init(&mut self) -> Result<(), EngineError> {
        let swapchain_device = swapchain::Device::new(self.get_instance()?, self.get_device()?);
        self.context.swapchain = Some(Swapchain {
            device: swapchain_device,
            handler: SwapchainKHR::default(),
            surface_format: SurfaceFormatKHR::default(),
            max_frames_in_flight: 0,
            images: Vec::new(),
            image_views: Vec::new(),
            depth_attachement: None,
        });

        self.swapchain_create(self.framebuffer_width, self.framebuffer_height)?;
        Ok(())
    }

    pub fn swapchain_shutdown(&mut self) -> Result<(), EngineError> {
        self.swapchain_destroy_base()?;
        self.context.swapchain = None;
        Ok(())
    }

    pub fn get_swapchain_next_image_index(
        &mut self,
        timeout_in_nanoseconds: u64,
        image_available_semaphore: Semaphore,
        fence: Fence,
    ) -> Result<Option<u32>, EngineError> {
        let swapchain = self.get_swapchain()?;
        unsafe {
            match swapchain.device.acquire_next_image(
                swapchain.handler,
                timeout_in_nanoseconds,
                image_available_semaphore,
                fence,
            ) {
                Ok((image_index, is_suboptimal)) => {
                    if is_suboptimal {
                        warn!("Found suboptimal swapchain when acquiring next image index: swapchain recreation...");
                        self.swapchain_recreate(self.framebuffer_width, self.framebuffer_height)?;
                        Ok(None)
                    } else {
                        Ok(Some(image_index))
                    }
                }
                Err(err) => {
                    if err == ash::vk::Result::ERROR_OUT_OF_DATE_KHR {
                        warn!("Found out of date swapchain when acquiring next image index: swapchain recreation...");
                        self.swapchain_recreate(self.framebuffer_width, self.framebuffer_height)?;
                        Ok(None)
                    } else {
                        error!(
                            "Failed to acquire the next vulkan swapchain image: {:?}",
                            err
                        );
                        Err(EngineError::VulkanFailed)
                    }
                }
            }
        }
    }

    pub fn present_swapchain(
        &mut self,
        render_complete_semaphore: Semaphore,
        present_image_index: u32,
    ) -> Result<(), EngineError> {
        let swapchain = self.get_swapchain()?;
        let wait_sempahores = [render_complete_semaphore];
        let swapchains = [swapchain.handler];
        let image_indices = [present_image_index];

        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_sempahores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let queues = self.get_queues()?;
        unsafe {
            match swapchain
                .device
                .queue_present(queues.present_queue.unwrap(), &present_info)
            {
                Ok(is_suboptimal) => {
                    if is_suboptimal {
                        warn!("Found suboptimal swapchain when presenting swapchain: swapchain recreation...");
                        self.swapchain_recreate(self.framebuffer_width, self.framebuffer_height)?;
                    };
                    Ok(())
                }
                Err(err) => {
                    if err == ash::vk::Result::ERROR_OUT_OF_DATE_KHR {
                        warn!("Found out of date swapchain when presenting swapchain: swapchain recreation...");
                        self.swapchain_recreate(self.framebuffer_width, self.framebuffer_height)?;
                        Ok(())
                    } else {
                        error!("Failed to present the vulkan swapchain image: {:?}", err);
                        Err(EngineError::VulkanFailed)
                    }
                }
            }
        }
    }

    pub fn get_swapchain(&self) -> Result<&Swapchain, EngineError> {
        match &self.context.swapchain {
            Some(swapchain) => Ok(swapchain),
            None => {
                error!("Can't access the vulkan swapchain");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
