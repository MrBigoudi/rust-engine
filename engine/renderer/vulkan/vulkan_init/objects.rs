use ash::vk::{BufferUsageFlags, MemoryPropertyFlags};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{
        vulkan_types::VulkanRendererBackend,
        vulkan_utils::buffer::{Buffer, BufferCreatorParameters},
    },
};

pub(crate) struct ObjectsBuffers {
    pub vertex_buffer: Buffer,
    pub vertex_offset: u64,

    pub index_buffer: Buffer,
    pub index_offset: u64,
}

impl VulkanRendererBackend<'_> {
    pub fn objects_buffers_init(&mut self) -> Result<(), EngineError> {
        let transfer_flags = BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::TRANSFER_SRC;
        // Vertex buffer
        let vertex_buffer_size = size_of::<glam::Vec3>() * 1024 * 1024;
        let vertex_buffer_creator_parameters = BufferCreatorParameters::default()
            .size(vertex_buffer_size)
            .buffer_usage_flags(transfer_flags | BufferUsageFlags::VERTEX_BUFFER)
            .memory_flags(MemoryPropertyFlags::DEVICE_LOCAL)
            .should_be_bind(true);
        let vertex_buffer = match self.create_buffer(vertex_buffer_creator_parameters) {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    "Failed to create the vertex buffer in the vulkan objects buffer: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        let vertex_offset = 0;

        // Index buffer
        let index_buffer_size = size_of::<u32>() * 1024 * 1024;
        let index_buffer_creator_parameters = BufferCreatorParameters::default()
            .size(index_buffer_size)
            .buffer_usage_flags(transfer_flags | BufferUsageFlags::INDEX_BUFFER)
            .memory_flags(MemoryPropertyFlags::DEVICE_LOCAL)
            .should_be_bind(true);
        let index_buffer = match self.create_buffer(index_buffer_creator_parameters) {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    "Failed to create the index buffer in the vulkan objects buffer: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        let index_offset = 0;

        self.context.objects = Some(ObjectsBuffers {
            vertex_buffer,
            index_buffer,
            vertex_offset,
            index_offset,
        });
        Ok(())
    }

    pub fn get_objects_buffers(&self) -> Result<&ObjectsBuffers, EngineError> {
        match &self.context.objects {
            Some(objects) => Ok(objects),
            None => {
                error!("Can't access the vulkan objects buffers");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn objects_buffers_shutdown(&mut self) -> Result<(), EngineError> {
        let objects_buffers = self.get_objects_buffers()?;
        if let Err(err) = self.destroy_buffer(&objects_buffers.index_buffer) {
            error!(
                "Failed to destroy the index buffer of the vulkan objects: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = self.destroy_buffer(&objects_buffers.vertex_buffer) {
            error!(
                "Failed to destroy the vertex buffer of the vulkan objects: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        Ok(())
    }
}
