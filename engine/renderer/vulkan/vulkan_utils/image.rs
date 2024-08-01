use ash::vk::{
    self, DeviceMemory, Extent3D, Format, ImageAspectFlags, ImageCreateInfo, ImageLayout,
    ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryPropertyFlags, SampleCountFlags, SharingMode,
};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

#[derive(Default)]
pub(crate) struct Image {
    pub memory: DeviceMemory,
    pub image: vk::Image,
    pub image_view: Option<ImageView>,
    pub width: u32,
    pub height: u32,
}

pub(crate) struct ImageCreatorParameters {
    pub image_type: ImageType,
    pub width: u32,
    pub height: u32,
    pub image_format: Format,
    pub image_tiling: ImageTiling,
    pub image_usage_flags: ImageUsageFlags,
    pub memory_flags: MemoryPropertyFlags,
    pub should_create_view: bool,
    pub image_view_aspect_flags: ImageAspectFlags,
}

impl ImageCreatorParameters {
    pub fn image_type(mut self, image_type: ImageType) -> Self {
        self.image_type = image_type;
        self
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }
    pub fn image_format(mut self, image_format: Format) -> Self {
        self.image_format = image_format;
        self
    }
    pub fn image_tiling(mut self, image_tiling: ImageTiling) -> Self {
        self.image_tiling = image_tiling;
        self
    }
    pub fn image_usage_flags(mut self, image_usage_flags: ImageUsageFlags) -> Self {
        self.image_usage_flags = image_usage_flags;
        self
    }
    pub fn memory_flags(mut self, memory_flags: MemoryPropertyFlags) -> Self {
        self.memory_flags = memory_flags;
        self
    }
    pub fn should_create_view(mut self, should_create_view: bool) -> Self {
        self.should_create_view = should_create_view;
        self
    }
    pub fn image_view_aspect_flags(mut self, image_view_aspect_flags: ImageAspectFlags) -> Self {
        self.image_view_aspect_flags = image_view_aspect_flags;
        self
    }
}

impl Default for ImageCreatorParameters {
    fn default() -> Self {
        Self {
            image_type: ImageType::TYPE_2D,
            width: 0,
            height: 0,
            image_format: Default::default(),
            image_tiling: Default::default(),
            image_usage_flags: Default::default(),
            memory_flags: Default::default(),
            should_create_view: Default::default(),
            image_view_aspect_flags: Default::default(),
        }
    }
}

impl VulkanRendererBackend<'_> {
    pub(crate) fn create_image(
        &self,
        image_creation_parameters: ImageCreatorParameters,
    ) -> Result<Image, EngineError> {
        let mut new_image = Image{
            width: image_creation_parameters.width,
            height: image_creation_parameters.height,
            ..Default::default()
        };

        // Creation info
        let image_create_info = ImageCreateInfo::default()
            .image_type(image_creation_parameters.image_type)
            .extent(Extent3D{ width: new_image.width, height: new_image.height, depth: 1 }) // TODO: Support configurable depth
            .mip_levels(4) // TODO: Support mip mapping
            .array_layers(1) // TODO: Support number of layer in the image
            .format(image_creation_parameters.image_format)
            .tiling(image_creation_parameters.image_tiling)
            .initial_layout(ImageLayout::UNDEFINED)
            .usage(image_creation_parameters.image_usage_flags)
            .samples(SampleCountFlags::TYPE_1) // TODO: Configurable sample count
            .sharing_mode(SharingMode::EXCLUSIVE) // TODO: Configurable sharing mode
        ;

        let device = &self.get_device()?;
        new_image.image = unsafe {
            match device.create_image(&image_create_info, self.get_allocator()?) {
                Ok(image) => image,
                Err(err) => {
                    error!("Failed to create a new vulkan image: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Query memory requirements
        let memory_requirements = unsafe { device.get_image_memory_requirements(new_image.image) };

        let memory_type = self.device_find_memory_index(
            memory_requirements.memory_type_bits,
            image_creation_parameters.memory_flags,
        )?;

        // Allocate memory
        let memory_allocate_info = MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type);

        new_image.memory = unsafe {
            match device.allocate_memory(&memory_allocate_info, self.get_allocator()?) {
                Ok(memory) => memory,
                Err(err) => {
                    error!(
                        "Failed to allocate memory for vulkan image creation: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Bind the memory
        unsafe {
            match device.bind_image_memory(new_image.image, new_image.memory, 0) {
                // TODO: configurable memory offset
                Ok(()) => (),
                Err(err) => {
                    error!(
                        "Failed to bind the image memory for vulkan image creation: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        }

        // Create image view
        if image_creation_parameters.should_create_view {
            new_image.image_view =
                match self.create_image_view(new_image.image, image_creation_parameters) {
                    Ok(image_view) => Some(image_view),
                    Err(err) => {
                        error!(
                            "Failed to create the image view for vulkan image creation: {:?}",
                            err
                        );
                        return Err(EngineError::VulkanFailed);
                    }
                };
        }

        Ok(new_image)
    }

    pub(crate) fn destroy_image(&self, image: &Image) -> Result<(), EngineError> {
        let device = self.get_device()?;

        if let Some(image_view) = image.image_view {
            unsafe {
                device.destroy_image_view(image_view, self.get_allocator()?);
            }
        }

        unsafe {
            device.free_memory(image.memory, self.get_allocator()?);
        }

        unsafe {
            device.destroy_image(image.image, self.get_allocator()?);
        }

        Ok(())
    }

    fn create_image_view(
        &self,
        image: vk::Image,
        image_creation_parameters: ImageCreatorParameters,
    ) -> Result<ImageView, EngineError> {
        // TODO: make the subresource configurable
        let image_subresource_range = ImageSubresourceRange::default()
            .aspect_mask(image_creation_parameters.image_view_aspect_flags)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let image_view_create_info = ImageViewCreateInfo::default()
            .image(image)
            .view_type(ImageViewType::TYPE_2D) // TODO: Make the view type configurable
            .format(image_creation_parameters.image_format)
            .subresource_range(image_subresource_range);

        let device = self.get_device()?;
        unsafe {
            match device.create_image_view(&image_view_create_info, self.get_allocator()?) {
                Ok(image_view) => Ok(image_view),
                Err(err) => {
                    error!("Failed to create a vulkan image view: {:?}", err);
                    Err(EngineError::InitializationFailed)
                }
            }
        }
    }
}
