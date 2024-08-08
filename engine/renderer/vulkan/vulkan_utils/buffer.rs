use std::ffi::c_void;

use ash::vk::{
    self, BufferCopy, BufferCreateInfo, BufferUsageFlags, CommandPool, DeviceMemory,
    MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, Queue, SharingMode,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{
        vulkan_init::command_buffer::CommandBuffer, vulkan_types::VulkanRendererBackend,
    },
};

#[derive(Default)]
pub(crate) struct Buffer {
    pub memory: DeviceMemory,
    pub buffer: vk::Buffer,
    pub total_size: usize,
    pub buffer_usage_flags: BufferUsageFlags,
    pub memory_flags: MemoryPropertyFlags,
}

pub(crate) struct CopyBufferParameters<'a> {
    pub src_buffer: &'a Buffer,
    pub src_offset: u64,
    pub dst_buffer: &'a Buffer,
    pub dst_offset: u64,
}

#[derive(Default)]
pub(crate) struct BufferCreatorParameters {
    pub size: usize,
    pub should_be_bind: bool,
    pub buffer_usage_flags: BufferUsageFlags,
    pub memory_flags: MemoryPropertyFlags,
}

impl BufferCreatorParameters {
    pub fn buffer_usage_flags(mut self, buffer_usage_flags: BufferUsageFlags) -> Self {
        self.buffer_usage_flags = buffer_usage_flags;
        self
    }
    pub fn memory_flags(mut self, memory_flags: MemoryPropertyFlags) -> Self {
        self.memory_flags = memory_flags;
        self
    }
    pub fn should_be_bind(mut self, should_be_bind: bool) -> Self {
        self.should_be_bind = should_be_bind;
        self
    }
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
}

impl VulkanRendererBackend<'_> {
    pub(crate) fn create_buffer(
        &self,
        buffer_creation_parameters: BufferCreatorParameters,
    ) -> Result<Buffer, EngineError> {
        // Creation info
        let buffer_create_info = BufferCreateInfo::default()
            .size(buffer_creation_parameters.size as u64)
            .usage(buffer_creation_parameters.buffer_usage_flags)
            .sharing_mode(SharingMode::EXCLUSIVE) // only used in one queue
        ;

        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        let buffer = unsafe {
            match device.create_buffer(&buffer_create_info, allocator) {
                Ok(buffer) => buffer,
                Err(err) => {
                    error!("Failed to create a vulkan buffer: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Gather memory requirements
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_index = match self.device_find_memory_index(
            memory_requirements.memory_type_bits,
            buffer_creation_parameters.memory_flags,
        ) {
            Ok(index) => index,
            Err(err) => {
                error!(
                    "Failed to find a memory index for a vulkan buffer creation: {:?}",
                    err
                );
                return Err(EngineError::VulkanFailed);
            }
        };
        // Allocate memory info
        let memory_allocate_info = MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_index);

        // Allocate the memory
        let memory = unsafe {
            match device.allocate_memory(&memory_allocate_info, allocator) {
                Ok(memory) => memory,
                Err(err) => {
                    error!("Failed to allocate a vulkan buffer memory: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        let new_buffer = Buffer {
            buffer,
            memory,
            buffer_usage_flags: buffer_creation_parameters.buffer_usage_flags,
            memory_flags: buffer_creation_parameters.memory_flags,
            total_size: buffer_creation_parameters.size,
        };

        if buffer_creation_parameters.should_be_bind {
            self.bind_buffer(&new_buffer, 0)?;
        }

        Ok(new_buffer)
    }

    pub(crate) fn bind_buffer(&self, buffer: &Buffer, offset: u64) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            if let Err(err) = device.bind_buffer_memory(buffer.buffer, buffer.memory, offset) {
                error!("Failed to bind a vulkan buffer: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }
        Ok(())
    }

    pub(crate) fn destroy_buffer(&self, buffer: &Buffer) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            device.free_memory(buffer.memory, allocator);
            device.destroy_buffer(buffer.buffer, allocator);
        }
        Ok(())
    }

    pub(crate) fn lock_memory_buffer(
        &self,
        buffer: &Buffer,
        offset: u64,
        size: usize,
        flags: MemoryMapFlags,
    ) -> Result<*mut c_void, EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            match device.map_memory(buffer.memory, offset, size as u64, flags) {
                Ok(data) => Ok(data),
                Err(err) => {
                    error!("Failed to lock the memory of a vulkan buffer: {:?}", err);
                    Err(EngineError::VulkanFailed)
                }
            }
        }
    }

    pub(crate) fn unlock_memory_buffer(&self, buffer: &Buffer) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        unsafe {
            device.unmap_memory(buffer.memory);
        }
        Ok(())
    }

    pub(crate) fn load_data_into_buffer(
        &self,
        buffer: &Buffer,
        offset: u64,
        size: usize,
        flags: MemoryMapFlags,
        data: *mut c_void,
    ) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        let space_in_memory = self.lock_memory_buffer(buffer, offset, size, flags)?;
        unsafe {
            space_in_memory.copy_from(data, size);
        }
        self.unlock_memory_buffer(buffer)?;
        Ok(())
    }

    pub(crate) fn copy_buffer_to(
        &self,
        command_pool: &CommandPool,
        queue: Queue,
        copy_parameters: CopyBufferParameters<'_>,
        size: usize,
    ) -> Result<(), EngineError> {
        self.device_wait_idle()?;
        let src_offset = copy_parameters.src_offset;
        let dst_offset = copy_parameters.dst_offset;
        let src_buffer = copy_parameters.src_buffer;
        let dst_buffer = copy_parameters.dst_buffer;

        // Create a one-time-use command buffer
        let device = self.get_device()?;
        let command_buffer = CommandBuffer::allocate_and_begin_single_use(device, command_pool)?;

        // Prepare the copy command and add it to the command buffer
        let copy_regions = [BufferCopy::default()
            .src_offset(src_offset)
            .dst_offset(dst_offset)
            .size(size as u64)];

        unsafe {
            device.cmd_copy_buffer(
                *command_buffer.handler.as_ref(),
                src_buffer.buffer,
                dst_buffer.buffer,
                &copy_regions,
            );
        }

        // Submit the buffer for execution and wait for it to complete
        command_buffer.end_single_use(device, command_pool, queue)?;
        Ok(())
    }

    pub(crate) fn resize_buffer(
        &self,
        buffer: Buffer,
        new_size: usize,
        queue: Queue,
        command_pool: &CommandPool,
    ) -> Result<Buffer, EngineError> {
        // Create new buffer
        let buffer_create_info = BufferCreateInfo::default()
            .size(new_size as u64)
            .usage(buffer.buffer_usage_flags)
            .sharing_mode(SharingMode::EXCLUSIVE);

        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        let new_buffer = unsafe {
            match device.create_buffer(&buffer_create_info, allocator) {
                Ok(buffer) => buffer,
                Err(err) => {
                    error!(
                        "Failed to recreate a buffer for vulkan buffer resizing: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Gather memory requirements
        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer.buffer) };
        let memory_index = self
            .device_find_memory_index(memory_requirements.memory_type_bits, buffer.memory_flags)?;
        // Allocate memory info
        let memory_allocate_info = MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_index);
        // Allocate the memory
        let new_memory = unsafe {
            match device.allocate_memory(&memory_allocate_info, allocator) {
                Ok(memory) => memory,
                Err(err) => {
                    error!("Failed to allocate a vulkan buffer memory for vulkan buffer resizing: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Bind the new buffer's memory
        unsafe {
            if let Err(err) = device.bind_buffer_memory(new_buffer, new_memory, 0) {
                error!(
                    "Failed to bind a vulkan buffer memory for vulkan buffer resizing: {:?}",
                    err
                );
                return Err(EngineError::VulkanFailed);
            }
        };

        // Copy over the data
        let new_buffer = Buffer {
            memory: new_memory,
            buffer: new_buffer,
            total_size: new_size,
            buffer_usage_flags: buffer.buffer_usage_flags,
            memory_flags: buffer.memory_flags,
        };
        let copy_parameters = CopyBufferParameters {
            src_buffer: &buffer,
            src_offset: 0,
            dst_buffer: &new_buffer,
            dst_offset: 0,
        };
        self.copy_buffer_to(command_pool, queue, copy_parameters, buffer.total_size)?;

        // Make sure anything potentially using these is finished
        self.device_wait_idle()?;

        // Destroy the old
        self.destroy_buffer(&buffer)?;

        Ok(new_buffer)
    }
}
