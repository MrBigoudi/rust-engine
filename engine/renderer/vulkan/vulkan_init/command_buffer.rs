use ash::{
    vk::{
        self, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
        CommandBufferResetFlags, CommandBufferUsageFlags, CommandPool, Fence, Queue, SubmitInfo,
    },
    Device,
};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

#[derive(Clone)]
pub(crate) struct CommandBuffer {
    pub handler: Box<vk::CommandBuffer>,
}

impl CommandBuffer {
    pub fn allocate(
        command_pool: &CommandPool,
        is_primary: bool,
        device: &Device,
    ) -> Result<Self, EngineError> {
        let command_buffer_info = CommandBufferAllocateInfo::default()
            .level(if is_primary {
                CommandBufferLevel::PRIMARY
            } else {
                CommandBufferLevel::SECONDARY
            })
            .command_buffer_count(1)
            .command_pool(*command_pool);

        let handler = unsafe {
            match device.allocate_command_buffers(&command_buffer_info) {
                Ok(command_buffer) => command_buffer[0],
                Err(err) => {
                    error!("Failed to allocate a vulkan command buffer: {:?}", err);
                    return Err(EngineError::InitializationFailed);
                }
            }
        };
        Ok(CommandBuffer {
            handler: Box::new(handler),
        })
    }

    pub fn free(&self, device: &Device, command_pool: &CommandPool) -> Result<(), EngineError> {
        let command_buffers = [*self.handler.as_ref()];
        unsafe {
            device.free_command_buffers(*command_pool, &command_buffers);
        }
        Ok(())
    }

    pub fn reset(&self, device: &Device) -> Result<(), EngineError> {
        unsafe {
            if let Err(err) = device
                .reset_command_buffer(*self.handler.as_ref(), CommandBufferResetFlags::empty())
            {
                error!("Failed to reset the command buffer: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }
        Ok(())
    }

    pub fn begin(
        &self,
        device: &Device,
        is_single_use: bool,
        is_renderpass_continue: bool,
        is_simultaneous_use: bool,
    ) -> Result<(), EngineError> {
        let mut command_buffer_info = CommandBufferBeginInfo::default();
        if is_single_use {
            command_buffer_info.flags |= CommandBufferUsageFlags::ONE_TIME_SUBMIT;
        }
        if is_renderpass_continue {
            command_buffer_info.flags |= CommandBufferUsageFlags::RENDER_PASS_CONTINUE;
        }
        if is_simultaneous_use {
            command_buffer_info.flags |= CommandBufferUsageFlags::SIMULTANEOUS_USE;
        }

        unsafe {
            if let Err(err) =
                device.begin_command_buffer(*self.handler.as_ref(), &command_buffer_info)
            {
                error!("Failed to begin a vulkan command buffer: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }

        Ok(())
    }

    pub fn end(&self, device: &Device) -> Result<(), EngineError> {
        unsafe {
            if let Err(err) = device.end_command_buffer(*self.handler.as_ref()) {
                error!("Failed to end a vulkan command buffer: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }
        Ok(())
    }

    pub fn allocate_and_begin_single_use(
        device: &Device,
        command_pool: &CommandPool,
    ) -> Result<CommandBuffer, EngineError> {
        let new_buffer = Self::allocate(command_pool, true, device)?;
        let is_single_use = true;
        let is_renderpass_continue = false;
        let is_simultaneous_use = false;
        new_buffer.begin(
            device,
            is_single_use,
            is_renderpass_continue,
            is_simultaneous_use,
        )?;
        Ok(new_buffer)
    }

    pub fn end_single_use(
        self,
        device: &Device,
        command_pool: &CommandPool,
        queue: Queue,
    ) -> Result<(), EngineError> {
        // End the command buffer.
        self.end(device)?;

        // Submit the queue
        let command_buffers = [*self.handler.as_ref()];
        let submit_info = [SubmitInfo::default().command_buffers(&command_buffers)];

        unsafe {
            if let Err(err) = device.queue_submit(queue, &submit_info, Fence::null()) {
                error!("Failed to submit a vulkan queue: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }

        // Wait for it to finish
        unsafe {
            if let Err(err) = device.queue_wait_idle(queue) {
                error!("Failed to wait fo a vulkan queue: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }

        // Free the command buffer.
        self.free(device, command_pool)?;

        Ok(())
    }
}

impl VulkanRendererBackend<'_> {
    pub fn graphics_command_buffers_shutdown(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let command_pool = self.get_graphics_command_pool()?;

        for buffer in &self.context.graphics_command_buffers {
            buffer.free(device, command_pool)?;
        }
        self.context.graphics_command_buffers.clear();
        Ok(())
    }

    pub fn graphics_command_buffers_init(&mut self) -> Result<(), EngineError> {
        // free the old command buffers
        self.graphics_command_buffers_shutdown()?;

        let nb_image_in_swapchain = self.get_swapchain()?.images.len();
        let command_pool = self.get_graphics_command_pool()?;
        let is_primary = true;
        let device = self.get_device()?;

        let mut new_buffers: Vec<CommandBuffer> = Vec::new();
        for index in 0..nb_image_in_swapchain {
            let new_buffer = CommandBuffer::allocate(command_pool, is_primary, device)?;
            new_buffers.push(new_buffer);
        }

        self.context.graphics_command_buffers = new_buffers;

        Ok(())
    }

    pub fn get_graphics_command_buffers(&self) -> Result<&Vec<CommandBuffer>, EngineError> {
        Ok(&self.context.graphics_command_buffers)
    }
}
