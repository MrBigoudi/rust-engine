use ash::{
    vk::{
        BufferUsageFlags, DescriptorBufferInfo, DescriptorPool, DescriptorPoolCreateInfo,
        DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout,
        DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, Extent2D,
        Format, MemoryMapFlags, MemoryPropertyFlags, Offset2D, PipelineBindPoint,
        PipelineShaderStageCreateInfo, Rect2D, ShaderStageFlags, VertexInputAttributeDescription,
        Viewport, WriteDescriptorSet,
    },
    Device,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::{
        renderer_types::RendererGlobalUniformObject,
        vulkan::{
            vulkan_init::command_buffer::CommandBuffer,
            vulkan_shaders::shader::Shader,
            vulkan_types::VulkanRendererBackend,
            vulkan_utils::{
                buffer::{Buffer, BufferCreatorParameters},
                pipeline::{Pipeline, PipelineCreateInfo},
            },
        },
    },
};

/// Default shader to display objects
pub(crate) struct ObjectShaders {
    pub vertex_stage: Shader,
    pub fragment_stage: Shader,
    pub pipeline: Pipeline,

    // One descriptor set per frame - max 3 for triple-buffering
    pub global_descriptor_sets: [DescriptorSet; 3],
    pub global_descriptor_pool: DescriptorPool,
    pub global_descriptor_set_layout: DescriptorSetLayout,
    pub global_ubo: RendererGlobalUniformObject,
    pub global_uniform_buffer: Buffer,
}

impl ObjectShaders {
    fn create_pipeline_info<'a>(
        backend: &'a VulkanRendererBackend<'a>,
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
        global_ubo_layout: DescriptorSetLayout,
    ) -> Result<PipelineCreateInfo<'a>, EngineError> {
        // Pipeline creation
        let viewports = vec![Viewport::default()
            .x(0.)
            .y(backend.framebuffer_height as f32)
            .width(backend.framebuffer_width as f32)
            .height(-(backend.framebuffer_height as f32))
            .min_depth(0.0)
            .max_depth(1.0)];

        // Scissor
        let scissors = vec![Rect2D::default()
            .offset(Offset2D { x: 0, y: 0 })
            .extent(Extent2D {
                width: backend.framebuffer_width,
                height: backend.framebuffer_height,
            })];

        // Input attributes
        let offset = 0;
        let position_attribute_description = VertexInputAttributeDescription::default()
            //  position
            .binding(0)// should match binding description
            .location(0)
            .format(Format::R32G32B32_SFLOAT)
            .offset(0) // because first, else offset += size_of::<attribute type>
        ;
        let vertex_input_attributes_description = vec![position_attribute_description];

        // Desciptor set layouts

        // Stages
        let shader_stages_info = vec![
            // vertex shader
            PipelineShaderStageCreateInfo::default()
                .stage(vertex_shader.stage_flag)
                .module(vertex_shader.shader_module)
                .name(vertex_shader.entry_point.as_c_str()),
            // fragment shader
            PipelineShaderStageCreateInfo::default()
                .stage(fragment_shader.stage_flag)
                .module(fragment_shader.shader_module)
                .name(fragment_shader.entry_point.as_c_str()),
        ];

        Ok(PipelineCreateInfo {
            renderpass: backend.get_renderpass()?,
            viewports,
            scissors,
            is_wireframe: false,
            vertex_input_attributes_description,
            descriptor_set_layouts: vec![global_ubo_layout],
            shader_stages_info,
        })
    }

    pub fn create(backend: &VulkanRendererBackend<'_>) -> Result<Self, EngineError> {
        let device = backend.get_device()?;
        let allocator = backend.get_allocator()?;

        // Shader module init per stage
        let vertex_stage = match Shader::create(
            device,
            allocator,
            ShaderStageFlags::VERTEX,
            "builtin/object.vert.slang",
            None,
        ) {
            Ok(shader) => shader,
            Err(err) => {
                error!("Failed to create the object vertex shader: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        };

        let fragment_stage = match Shader::create(
            device,
            allocator,
            ShaderStageFlags::FRAGMENT,
            "builtin/object.frag.slang",
            None,
        ) {
            Ok(shader) => shader,
            Err(err) => {
                error!("Failed to create the object fragment shader: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        };

        // Descriptors
        // Global Descriptors
        let global_ubo_layout_bindings = [DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .stage_flags(ShaderStageFlags::VERTEX)];
        let global_ubo_layout_create_info =
            DescriptorSetLayoutCreateInfo::default().bindings(&global_ubo_layout_bindings);
        let device = backend.get_device()?;
        let allocator = backend.get_allocator()?;
        let global_ubo_layout = unsafe {
            match device.create_descriptor_set_layout(&global_ubo_layout_create_info, allocator) {
                Ok(layout) => layout,
                Err(err) => {
                    error!("Failed to create a vulkan global uniform buffer: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };
        // Global descriptor pool: Used for global items such as view/projection matrix
        let image_count = backend.get_swapchain()?.images.len() as u32;
        let global_descriptor_pool_sizes = [DescriptorPoolSize::default()
            .ty(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(image_count)];
        let global_descriptor_pool_create_info = DescriptorPoolCreateInfo::default()
            .pool_sizes(&global_descriptor_pool_sizes)
            .max_sets(image_count);
        let global_descriptor_pool = unsafe {
            match device.create_descriptor_pool(&global_descriptor_pool_create_info, allocator) {
                Ok(pool) => pool,
                Err(err) => {
                    error!(
                        "Failed to create a vulkan global descriptor pool: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Pipelines
        let pipeline_info = match Self::create_pipeline_info(
            backend,
            &vertex_stage,
            &fragment_stage,
            global_ubo_layout,
        ) {
            Ok(info) => info,
            Err(err) => {
                error!(
                    "Failed to create the pipeline info when creating vulkan object shaders: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        let pipeline = match Pipeline::create_graphics(device, allocator, pipeline_info) {
            Ok(pipeline) => pipeline,
            Err(err) => {
                error!(
                    "Failed to create the pipeline when creating vulkan object shaders: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        // Create uniform buffer
        let global_uniform_buffer_creator_params = BufferCreatorParameters::default()
            .buffer_usage_flags(BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::UNIFORM_BUFFER)
            .memory_flags(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT)
            .should_be_bind(true)
            .size(size_of::<RendererGlobalUniformObject>());
        let global_uniform_buffer = match backend
            .create_buffer(global_uniform_buffer_creator_params)
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!("Failed to create the global uniform buffer when creating vulkan object shaders: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        };

        // Allocate global descriptor sets
        let global_descriptor_sets_layouts =
            [global_ubo_layout, global_ubo_layout, global_ubo_layout];
        let global_descriptor_sets_allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(global_descriptor_pool)
            .set_layouts(&global_descriptor_sets_layouts);
        let global_descriptor_sets = unsafe {
            match device.allocate_descriptor_sets(&global_descriptor_sets_allocate_info) {
                Ok(sets) => sets,
                Err(err) => {
                    error!("Failed to create a vulkan global descriptor set: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };
        let global_descriptor_sets = [
            global_descriptor_sets[0],
            global_descriptor_sets[1],
            global_descriptor_sets[2],
        ];

        Ok(ObjectShaders {
            vertex_stage,
            fragment_stage,
            pipeline,
            global_descriptor_pool,
            global_descriptor_set_layout: global_ubo_layout,
            global_descriptor_sets,
            global_ubo: RendererGlobalUniformObject::default(),
            global_uniform_buffer,
        })
    }

    pub fn destroy(&self, backend: &VulkanRendererBackend<'_>) -> Result<(), EngineError> {
        let device = backend.get_device()?;
        let allocator = backend.get_allocator()?;

        // Destroy uniform buffer
        if let Err(err) = backend.destroy_buffer(&self.global_uniform_buffer) {
            error!(
                "Failed to destroy the global uniform buffer of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = self.pipeline.destroy(device, allocator) {
            error!(
                "Failed to destroy the pipeline of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = self.vertex_stage.destroy(device, allocator) {
            error!(
                "Failed to destroy the vertex stage of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = self.fragment_stage.destroy(device, allocator) {
            error!(
                "Failed to destroy the fragment stage of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        unsafe {
            device.destroy_descriptor_pool(self.global_descriptor_pool, allocator);
            device.destroy_descriptor_set_layout(self.global_descriptor_set_layout, allocator);
        }
        Ok(())
    }

    pub fn r#use(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
    ) -> Result<(), EngineError> {
        let pipeline = &self.pipeline;
        if let Err(err) = pipeline.bind(device, command_buffer, PipelineBindPoint::GRAPHICS) {
            error!(
                "Failed to bind the pipeline of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        Ok(())
    }
}

impl VulkanRendererBackend<'_> {
    pub fn update_object_shaders_global_state(&mut self) -> Result<(), EngineError> {
        let current_frame_index = self.context.current_frame as usize;
        let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
        let device = self.get_device()?;

        // Bind the global descriptor set to be updated
        let object_shaders = &self.get_builtin_shaders()?.object_shaders;
        let global_descriptor_set = [object_shaders.global_descriptor_sets[current_frame_index]];
        unsafe {
            let offsets = [];
            device.cmd_bind_descriptor_sets(
                *command_buffer.handler.as_ref(),
                PipelineBindPoint::GRAPHICS,
                object_shaders.pipeline.layout,
                0,
                &global_descriptor_set,
                &offsets,
            );
        }

        // Configure the descriptors for the given index
        let range = size_of::<RendererGlobalUniformObject>();
        let offset = 0;

        // Copy data to buffer
        let data = {
            let object_shaders = &mut self
                .context
                .builtin_shaders
                .as_mut()
                .unwrap()
                .object_shaders;
            &mut object_shaders.global_ubo as *mut RendererGlobalUniformObject
                as *mut std::ffi::c_void
        };

        let object_shaders = &self.get_builtin_shaders()?.object_shaders;
        if let Err(err) = self.load_data_into_buffer(
            &object_shaders.global_uniform_buffer,
            offset,
            range,
            MemoryMapFlags::empty(),
            data,
        ) {
            error!("Failed to load the global uniform data to the global uniform buffer when updating the state of the vulkan object shaders: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }

        let descriptor_buffer_info = [DescriptorBufferInfo::default()
            .buffer(object_shaders.global_uniform_buffer.buffer)
            .offset(offset)
            .range(range as u64)];

        // Update descriptor sets
        let descriptor_writes = [WriteDescriptorSet::default()
            .dst_set(object_shaders.global_descriptor_sets[current_frame_index])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .buffer_info(&descriptor_buffer_info)];
        let descriptor_copies = [];

        let device = self.get_device()?;
        unsafe { device.update_descriptor_sets(&descriptor_writes, &descriptor_copies) };

        Ok(())
    }
}
