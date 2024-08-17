use ash::vk::{
    BorderColor, BufferUsageFlags, CompareOp, Filter, Format, ImageAspectFlags, ImageLayout,
    ImageTiling, ImageType, ImageUsageFlags, MemoryMapFlags, MemoryPropertyFlags, Sampler,
    SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{
        vulkan_init::command_buffer::CommandBuffer, vulkan_types::VulkanRendererBackend,
    },
    resources::texture::TextureCreatorParameters,
};

use super::{
    buffer::BufferCreatorParameters,
    image::{Image, ImageCreatorParameters},
};

#[derive(Clone, Copy)]
pub(crate) struct Texture {
    pub width: u32,
    pub height: u32,
    pub id: u32,
    pub nb_channels: u8,
    pub generation: Option<u32>,
    pub has_transparency: bool,
    pub image: Image,
    pub sampler: Sampler,
}

impl crate::resources::texture::Texture for Texture {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_width(&self) -> u32 {
        self.width
    }

    fn get_height(&self) -> u32 {
        self.height
    }

    fn get_nb_channels(&self) -> u8 {
        self.nb_channels
    }

    fn has_transparency(&self) -> bool {
        self.has_transparency
    }

    fn get_generation(&self) -> Option<u32> {
        self.generation
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn clone_box(&self) -> Box<dyn crate::resources::texture::Texture> {
        Box::new(*self)
    }
}

impl VulkanRendererBackend<'_> {
    pub(crate) fn vulkan_destroy_texture(&self, texture: &Texture) -> Result<(), EngineError> {
        if let Err(err) = self.device_wait_idle() {
            error!(
                "Failed to wait idle when destroying a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = self.destroy_image(&texture.image) {
            error!(
                "Failed to destroy the image when destroying a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }

        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            device.destroy_sampler(texture.sampler, allocator);
        }
        Ok(())
    }

    pub(crate) fn vulkan_create_texture(
        &self,
        params: TextureCreatorParameters,
    ) -> Result<Texture, EngineError> {
        // Internal data creation
        // Create a staging buffer and load data into it
        let image_size = (params.width * params.height * (params.nb_channels as u32)) as usize;
        let memory_prop_flags =
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT;
        let buffer_create_info = BufferCreatorParameters::default()
            .buffer_usage_flags(BufferUsageFlags::TRANSFER_SRC)
            .memory_flags(memory_prop_flags)
            .size(image_size)
            .should_be_bind(true);
        let staging = match self.create_buffer(buffer_create_info) {
            Ok(staging) => staging,
            Err(err) => {
                error!(
                    "Failed to create a stagging buffer when creating a vulkan texture: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        let data = params.pixels;
        let data = data.as_ptr() as *mut std::ffi::c_void;
        if let Err(err) =
            self.load_data_into_buffer(&staging, 0, image_size, MemoryMapFlags::empty(), data)
        {
            error!(
                "Failed to load data into a stagging buffer when creating a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        };

        // NOTE: Lots of assumptions here, different texture types will require different options here
        // NOTE: Assumes 8 bits per channel
        let image_format = Format::R8G8B8A8_UNORM;
        let image_create_info = ImageCreatorParameters::default()
            .width(params.width)
            .height(params.height)
            .image_type(ImageType::TYPE_2D)
            .image_format(image_format)
            .image_tiling(ImageTiling::OPTIMAL)
            .memory_flags(MemoryPropertyFlags::DEVICE_LOCAL)
            .image_usage_flags(
                ImageUsageFlags::TRANSFER_SRC
                    | ImageUsageFlags::TRANSFER_DST
                    | ImageUsageFlags::SAMPLED
                    | ImageUsageFlags::COLOR_ATTACHMENT,
            )
            .should_create_view(true)
            .image_view_aspect_flags(ImageAspectFlags::COLOR);
        let image = match self.create_image(image_create_info) {
            Ok(image) => image,
            Err(err) => {
                error!(
                    "Failed to create an image when creating a vulkan texture: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        let pool = self.get_graphics_command_pool()?;
        let device = self.get_device()?;
        let temporary_buffer = match CommandBuffer::allocate_and_begin_single_use(device, pool) {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    "Failed to allocate a staging buffer when creating a vulkan texture: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        // Transition the layout from whatever it is currently to optimal for recieving data
        if let Err(err) = self.transition_image_layout(
            &temporary_buffer,
            &image,
            image_format,
            ImageLayout::UNDEFINED,
            ImageLayout::TRANSFER_DST_OPTIMAL,
        ) {
            error!(
                "Failed to transition the image layout when creating a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        // Copy the data from the buffer
        if let Err(err) = self.copy_image_from_buffer(&temporary_buffer, &staging, &image) {
            error!("Failed to copy the image from the staging buffer when creating a vulkan texture: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }

        // Transition from optimal for data reciept to shader-read-only optimal layout
        if let Err(err) = self.transition_image_layout(
            &temporary_buffer,
            &image,
            image_format,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        ) {
            error!(
                "Failed to transition the image layout when creating a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        let device = self.get_device()?;
        let queue = self.get_queues()?.graphics_queue.unwrap();
        if let Err(err) = temporary_buffer.end_single_use(device, pool, queue) {
            error!("Failed to end the single use of the staging buffer when creating a vulkan texture: {:?}", err);
            return Err(EngineError::InitializationFailed);
        }

        // Create a sampler for the texture
        // TODO: These filters should be configurable.
        let sampler_create_info = SamplerCreateInfo::default()
            .mag_filter(Filter::LINEAR)
            .min_filter(Filter::LINEAR)
            .address_mode_u(SamplerAddressMode::REPEAT)
            .address_mode_v(SamplerAddressMode::REPEAT)
            .address_mode_w(SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(CompareOp::ALWAYS)
            .mipmap_mode(SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        let allocator = self.get_allocator()?;
        let sampler = unsafe {
            match device.create_sampler(&sampler_create_info, allocator) {
                Ok(sampler) => sampler,
                Err(err) => {
                    error!(
                        "Failed to create a texture sampler when creating a vulkan texture: {:?}",
                        err
                    );
                    return Err(EngineError::InitializationFailed);
                }
            }
        };

        // Destroy the staging buffer
        if let Err(err) = self.destroy_buffer(&staging) {
            error!(
                "Failed to destroy the staging buffer when creating a vulkan texture: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }

        let generation = if params.is_default { None } else { Some(0) };

        Ok(Texture {
            width: params.width,
            height: params.height,
            id: 0, // TODO: change id
            nb_channels: params.nb_channels,
            generation,
            has_transparency: params.has_transparency,
            image,
            sampler,
        })
    }
}
