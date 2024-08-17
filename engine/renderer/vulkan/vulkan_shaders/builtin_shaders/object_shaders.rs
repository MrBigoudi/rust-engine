use ash::{
    vk::{
        BufferUsageFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool,
        DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo,
        DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
        DescriptorType, Extent2D, Format, ImageLayout, MemoryMapFlags, MemoryPropertyFlags,
        Offset2D, PipelineBindPoint, PipelineShaderStageCreateInfo, Rect2D, ShaderStageFlags,
        VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, Viewport,
        WriteDescriptorSet,
    },
    Device,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::{
        renderer_frontend::renderer_get_default_texture,
        renderer_types::{
            GeometryRenderData, RendererGlobalUniformObject, RendererPerObjectUniformObject,
            RENDERER_MAX_IN_FLIGHT_FRAMES,
        },
        vulkan::{
            vulkan_init::command_buffer::CommandBuffer,
            vulkan_shaders::shader::Shader,
            vulkan_types::VulkanRendererBackend,
            vulkan_utils::{
                buffer::{Buffer, BufferCreatorParameters},
                pipeline::{Pipeline, PipelineCreateInfo},
                texture::Texture,
            },
        },
    },
};

pub const VULKAN_MAX_OBJECT_COUNT: usize = 1024;
pub const VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT: usize = 2;

#[derive(Default, Clone, Copy)]
pub(crate) struct DescriptorState {
    // One per frame
    pub generations: [Option<u32>; RENDERER_MAX_IN_FLIGHT_FRAMES],
}

#[derive(Default, Clone, Copy)]
pub(crate) struct ObjectShadersPerObjectState {
    // Per frame
    pub descriptor_sets: [DescriptorSet; RENDERER_MAX_IN_FLIGHT_FRAMES],
    // Per descriptor
    pub descriptor_states: [DescriptorState; VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT],
}

/// Default shader to display objects
pub(crate) struct ObjectShaders {
    pub vertex_stage: Shader,
    pub fragment_stage: Shader,
    pub pipeline: Pipeline,

    // One descriptor set per frame
    pub global_descriptor_sets: [DescriptorSet; RENDERER_MAX_IN_FLIGHT_FRAMES],
    pub global_descriptor_pool: DescriptorPool,
    pub global_descriptor_set_layout: DescriptorSetLayout,
    pub global_ubo: RendererGlobalUniformObject,
    pub global_uniform_buffer: Buffer,

    pub per_object_descriptor_pool: DescriptorPool,
    pub per_object_descriptor_set_layout: DescriptorSetLayout,
    pub per_object_uniform_buffer: Buffer,
    // TODO: manage a free list of some kind here instead
    pub object_uniform_buffer_index: u32,
    // TODO: make dynamic
    pub object_states: [ObjectShadersPerObjectState; VULKAN_MAX_OBJECT_COUNT],
}

impl ObjectShaders {
    fn create_pipeline_info<'a>(
        backend: &'a VulkanRendererBackend<'a>,
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
        layouts: Vec<DescriptorSetLayout>,
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
        let vertex_input_binding_description = VertexInputBindingDescription::default()
            .binding(0)
            .stride((size_of::<glam::Vec3>() + size_of::<glam::Vec2>()) as u32)
            .input_rate(VertexInputRate::VERTEX);
        let position_attribute_description = VertexInputAttributeDescription::default()
            //  position
            .binding(vertex_input_binding_description.binding)// should match binding description
            .location(0)
            .format(Format::R32G32B32_SFLOAT)
            .offset(0) // because first, else offset += size_of::<attribute type>
        ;
        let texture_attribute_description = VertexInputAttributeDescription::default()
            //  texture coordinates
            .binding(vertex_input_binding_description.binding)// should match binding description
            .location(1)
            .format(Format::R32G32_SFLOAT)
            .offset(size_of::<glam::Vec3>() as u32) // offset += size_of::<previous attribute type>
        ;
        let vertex_input_attributes_description = vec![
            position_attribute_description,
            texture_attribute_description,
        ];
        let vertex_input_bindings_description = vec![vertex_input_binding_description];

        // descriptor set layouts
        let descriptor_set_layouts = layouts;

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
            vertex_input_bindings_description,
            descriptor_set_layouts,
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

        // Local/Object Descriptors
        let local_sampler_count = 1;
        let local_descriptor_types: [DescriptorType;
            VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT] = [
            DescriptorType::UNIFORM_BUFFER,         // Binding 0 - uniform buffer
            DescriptorType::COMBINED_IMAGE_SAMPLER, // Binding 1 - Diffuse sampler layout
        ];
        let mut local_descriptor_set_layout_bindings: [DescriptorSetLayoutBinding;
            VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT] =
            [DescriptorSetLayoutBinding::default()
                .descriptor_count(1)
                .stage_flags(ShaderStageFlags::FRAGMENT);
                VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT];
        for (i, val) in local_descriptor_set_layout_bindings.iter_mut().enumerate() {
            val.binding = i as u32;
            val.descriptor_type = local_descriptor_types[i];
        }

        let local_descriptor_set_layout_create_info = DescriptorSetLayoutCreateInfo::default()
            .bindings(&local_descriptor_set_layout_bindings);
        let local_descriptor_set_layouts = unsafe {
            match device
                .create_descriptor_set_layout(&local_descriptor_set_layout_create_info, allocator)
            {
                Ok(layouts) => layouts,
                Err(err) => {
                    error!("Failed to create the local descriptor layouts of the vulkan object shaders: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Local/Object descriptor pool: Used for object-specific items like diffuse colour
        let local_descriptor_pool_sizes: [DescriptorPoolSize;
            VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT] = [
            // The first section will be used for uniform buffers
            DescriptorPoolSize::default()
                .ty(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(VULKAN_MAX_OBJECT_COUNT as u32),
            // The second section will be used for image samplers
            DescriptorPoolSize::default()
                .ty(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(local_sampler_count * VULKAN_MAX_OBJECT_COUNT as u32),
        ];
        let local_descriptor_pool_create_info = DescriptorPoolCreateInfo::default()
            .pool_sizes(&local_descriptor_pool_sizes)
            .max_sets(VULKAN_MAX_OBJECT_COUNT as u32);

        // Create object descriptor pool
        let local_descriptor_pool = unsafe {
            match device.create_descriptor_pool(&local_descriptor_pool_create_info, allocator) {
                Ok(pool) => pool,
                Err(err) => {
                    error!("failed to create the local descriptor pool of the vulkan object shaders: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Descriptor layouts
        let layouts = vec![global_ubo_layout, local_descriptor_set_layouts];

        // Pipelines
        let pipeline_info =
            match Self::create_pipeline_info(backend, &vertex_stage, &fragment_stage, layouts) {
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

        // Create the local uniform buffer
        let local_uniform_buffer_creator_params = BufferCreatorParameters::default()
            .buffer_usage_flags(BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::UNIFORM_BUFFER)
            .memory_flags(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT)
            .should_be_bind(true)
            .size(size_of::<RendererPerObjectUniformObject>());
        let local_uniform_buffer = match backend.create_buffer(local_uniform_buffer_creator_params)
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!("Failed to create the local uniform buffer when creating vulkan object shaders: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
        };

        Ok(ObjectShaders {
            vertex_stage,
            fragment_stage,
            pipeline,
            global_descriptor_pool,
            global_descriptor_set_layout: global_ubo_layout,
            global_descriptor_sets,
            global_ubo: RendererGlobalUniformObject::default(),
            global_uniform_buffer,
            per_object_descriptor_pool: local_descriptor_pool,
            per_object_descriptor_set_layout: local_descriptor_set_layouts,
            per_object_uniform_buffer: local_uniform_buffer,
            object_uniform_buffer_index: 0,
            object_states: [ObjectShadersPerObjectState::default(); VULKAN_MAX_OBJECT_COUNT],
        })
    }

    pub fn destroy(&self, backend: &VulkanRendererBackend<'_>) -> Result<(), EngineError> {
        let device = backend.get_device()?;
        let allocator = backend.get_allocator()?;

        // Destroy uniform buffers
        if let Err(err) = backend.destroy_buffer(&self.global_uniform_buffer) {
            error!(
                "Failed to destroy the global uniform buffer of the vulkan object shaders: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        if let Err(err) = backend.destroy_buffer(&self.per_object_uniform_buffer) {
            error!(
                "Failed to destroy the per object uniform buffer of the vulkan object shaders: {:?}",
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
            device.destroy_descriptor_pool(self.per_object_descriptor_pool, allocator);
            device.destroy_descriptor_set_layout(self.per_object_descriptor_set_layout, allocator);
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
        let delta_time = self.frame_delta_time;

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

    pub fn update_object_shaders(&mut self, data: &GeometryRenderData) -> Result<(), EngineError> {
        let current_frame_index = self.context.current_frame as usize;
        let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
        let device = self.get_device()?;
        let object_shaders = &self.get_builtin_shaders()?.object_shaders;

        // Convert glam::Mat4 into &[u8]
        let ptr: *const glam::Mat4 = &data.model;
        // Convert the raw pointer to a raw pointer to u8
        let byte_ptr: *const u8 = ptr as *const u8;
        // Calculate the length of the byte slice (Mat4 is 16 floats, each 4 bytes)
        let len = size_of::<glam::Mat4>();
        // Convert the raw pointer into a byte slice
        let constants = unsafe { std::slice::from_raw_parts(byte_ptr, len) };

        unsafe {
            device.cmd_push_constants(
                *command_buffer.handler.as_ref(),
                object_shaders.pipeline.layout,
                ShaderStageFlags::VERTEX,
                0,
                constants,
            );
        }

        // Obtain material data
        let object_id = match data.object_id {
            Some(id) => id as usize,
            None => {
                error!("The object id is none");
                return Err(EngineError::InvalidValue);
            }
        };

        let state: &ObjectShadersPerObjectState = match object_shaders.object_states.get(object_id)
        {
            Some(_) => &object_shaders.object_states[object_id],
            None => {
                error!("The state does not exist");
                return Err(EngineError::InvalidValue);
            }
        };

        let object_descriptor_set = state.descriptor_sets[current_frame_index];

        // TODO: if needs update
        let mut write_descriptors: Vec<WriteDescriptorSet> = Vec::new();

        // Descriptor 0 - Uniform buffer
        let range = size_of::<RendererPerObjectUniformObject>();
        let offset = (size_of::<RendererPerObjectUniformObject>() * object_id) as u64; // also the index into the array.

        // TODO: get diffuse colour from a material
        let diffuse = glam::Vec4::new(1.0, 1.0, 1.0, 1.0);

        // buffer
        let mut object_uniform_buffer = RendererPerObjectUniformObject::default().diffuse(diffuse);
        let object_uniform_buffer = &mut object_uniform_buffer
            as *mut RendererPerObjectUniformObject
            as *mut std::ffi::c_void;

        // Load the data into the buffer
        if let Err(err) = self.load_data_into_buffer(
            &object_shaders.per_object_uniform_buffer,
            offset,
            range,
            MemoryMapFlags::empty(),
            object_uniform_buffer,
        ) {
            error!(
                "Failed to load data into buffers when updating objects shader: {:?}",
                err
            );
            return Err(EngineError::Unknown);
        }

        // Only do this if the descriptor has not yet been updated
        let mut descriptor_index = 0;
        let mut should_update_descriptor_sets = false;

        let descriptor_buffer_info_tmp = [DescriptorBufferInfo::default()
            .buffer(object_shaders.per_object_uniform_buffer.buffer)
            .offset(offset)
            .range(range as u64)];
        if state.descriptor_states[descriptor_index].generations[current_frame_index].is_none() {
            let descriptor = WriteDescriptorSet::default()
                .dst_set(object_descriptor_set)
                .dst_binding(descriptor_index as u32)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(&descriptor_buffer_info_tmp);
            write_descriptors.push(descriptor);
            should_update_descriptor_sets = true;

            // Update the frame generation. In this case it is only needed once since this is a buffer
            let object_shaders = &mut self
                .context
                .builtin_shaders
                .as_mut()
                .unwrap()
                .object_shaders;
            let state: &mut ObjectShadersPerObjectState =
                match object_shaders.object_states.get(object_id) {
                    Some(_) => &mut object_shaders.object_states[object_id],
                    None => {
                        error!("The state does not exist");
                        return Err(EngineError::InvalidValue);
                    }
                };
            state.descriptor_states[descriptor_index].generations[current_frame_index] = Some(1);
        }
        descriptor_index += 1;

        // TODO: other samplers
        let sampler_count = 1; // only one texture for now
        let mut descriptor_image_info_tmp: Vec<(
                [DescriptorImageInfo; 1], // descriptor_image_info
                u32,                      // descriptor_index,
            )> = Vec::new()
        ;
        for sampler_index in 0..sampler_count {
            // for sampler_index in 0..sampler_count {
            let object_shaders = &self.get_builtin_shaders()?.object_shaders;
            let state: &ObjectShadersPerObjectState =
                match object_shaders.object_states.get(object_id) {
                    Some(_) => &object_shaders.object_states[object_id],
                    None => {
                        error!("The state does not exist");
                        return Err(EngineError::InvalidValue);
                    }
                };
            let texture = &data.textures[sampler_index];
            let generation =
                state.descriptor_states[descriptor_index].generations[current_frame_index];

            if let Some(texture) = texture {
                // If the texture hasn't been loaded yet, use the default
                // TODO: Determine which use the texture has and pull appropriate default based on that
                let (texture, is_default_texture) = if texture.get_generation().is_none() {
                    // Reset the descriptor generation if using the default texture
                    let object_shaders = &mut self
                        .context
                        .builtin_shaders
                        .as_mut()
                        .unwrap()
                        .object_shaders;
                    let state: &mut ObjectShadersPerObjectState =
                        match object_shaders.object_states.get(object_id) {
                            Some(_) => &mut object_shaders.object_states[object_id],
                            None => {
                                error!("The state does not exist");
                                return Err(EngineError::InvalidValue);
                            }
                        };
                    state.descriptor_states[descriptor_index].generations[current_frame_index] =
                        None;
                    (
                        match renderer_get_default_texture() {
                            Ok(texture) => texture,
                            Err(err) => {
                                error!("Failed to fetch the default texture when updating the object shaders: {:?}", err);
                                return Err(EngineError::AccessFailed);
                            }
                        },
                        true,
                    )
                } else {
                    (texture.as_ref(), false)
                };
                // Check if the descriptor needs updating first
                if texture.get_generation() != generation || is_default_texture {
                    let vulkan_texture = match texture.as_any().downcast_ref::<Texture>() {
                        Some(texture) => texture,
                        None => {
                            error!("Failed to downcast a texture to a vulkan texture");
                            return Err(EngineError::InvalidValue);
                        }
                    };

                    // assign view and sampler
                    let descriptor_image_info = DescriptorImageInfo::default()
                        .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .image_view(vulkan_texture.image.image_view.unwrap())
                        .sampler(vulkan_texture.sampler);

                    descriptor_image_info_tmp.push(
                        (
                            [descriptor_image_info], 
                            descriptor_index as u32
                        )
                    );

                    should_update_descriptor_sets = true;
                    
                    // Sync frame generation if not using a default texture
                    if texture.get_generation().is_some() {
                        let object_shaders = &mut self
                            .context
                            .builtin_shaders
                            .as_mut()
                            .unwrap()
                            .object_shaders;
                        let state: &mut ObjectShadersPerObjectState =
                            match object_shaders.object_states.get(object_id) {
                                Some(_) => &mut object_shaders.object_states[object_id],
                                None => {
                                    error!("The state does not exist");
                                    return Err(EngineError::InvalidValue);
                                }
                            };
                        state.descriptor_states[descriptor_index].generations
                            [current_frame_index] = texture.get_generation();
                    }
                    descriptor_index += 1;
                }
            }
        }
        for (descriptor_image_info, descriptor_index) in &descriptor_image_info_tmp {
            let descriptor = WriteDescriptorSet::default()
                .dst_set(object_descriptor_set)
                .dst_binding(*descriptor_index)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .image_info(descriptor_image_info);
            write_descriptors.push(descriptor);
        }

        let device = self.get_device()?;
        if should_update_descriptor_sets {
            unsafe {
                device.update_descriptor_sets(&write_descriptors, &[]);
            }
        }

        // Bind the descriptor set to be updated, or in case the shader changed
        let sets = [object_descriptor_set];
        let object_shaders = &self.get_builtin_shaders()?.object_shaders;
        let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
        unsafe {
            device.cmd_bind_descriptor_sets(
                *command_buffer.handler.as_ref(),
                PipelineBindPoint::GRAPHICS,
                object_shaders.pipeline.layout,
                1,
                &sets,
                &[],
            );
        }

        Ok(())
    }

    /// Returns the object id of the new resource
    pub fn object_shader_acquire_resources(&mut self) -> Result<u32, EngineError> {
        // TODO: free list
        let object_shaders = &self.get_builtin_shaders()?.object_shaders;
        let object_id = object_shaders.object_uniform_buffer_index;
        let object_shaders = &mut self
            .context
            .builtin_shaders
            .as_mut()
            .unwrap()
            .object_shaders;
        object_shaders.object_uniform_buffer_index += 1;

        let state: &mut ObjectShadersPerObjectState =
            match object_shaders.object_states.get(object_id as usize) {
                Some(_) => &mut object_shaders.object_states[object_id as usize],
                None => {
                    error!("The state does not exist");
                    return Err(EngineError::InvalidValue);
                }
            };
        for i in 0..VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT {
            for j in 0..RENDERER_MAX_IN_FLIGHT_FRAMES {
                state.descriptor_states[i].generations[j] = None;
            }
        }

        // Allocate descriptor sets
        let layouts =
            [object_shaders.per_object_descriptor_set_layout; RENDERER_MAX_IN_FLIGHT_FRAMES];
        let allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(object_shaders.per_object_descriptor_pool)
            .set_layouts(&layouts);
        let device = self.get_device()?;
        let descriptor_sets = unsafe {
            match device.allocate_descriptor_sets(&allocate_info) {
                Ok(set) => set,
                Err(err) => {
                    error!("Failed to allocate descriptor set: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        if descriptor_sets.len() != RENDERER_MAX_IN_FLIGHT_FRAMES {
            error!("The descriptor doesn't have the required number of elements");
            return Err(EngineError::InvalidValue);
        }
        let object_shaders = &mut self
            .context
            .builtin_shaders
            .as_mut()
            .unwrap()
            .object_shaders;
        let state: &mut ObjectShadersPerObjectState =
            match object_shaders.object_states.get(object_id as usize) {
                Some(_) => &mut object_shaders.object_states[object_id as usize],
                None => {
                    error!("The state does not exist");
                    return Err(EngineError::InvalidValue);
                }
            };
        state.descriptor_sets[..RENDERER_MAX_IN_FLIGHT_FRAMES]
            .copy_from_slice(&descriptor_sets[..RENDERER_MAX_IN_FLIGHT_FRAMES]);

        Ok(object_id)
    }

    pub fn object_shader_release_resources(&mut self, object_id: u32) -> Result<(), EngineError> {
        let object_shaders = &self
            .context
            .builtin_shaders
            .as_ref()
            .unwrap()
            .object_shaders;
        let state = match object_shaders.object_states.get(object_id as usize) {
            Some(_) => &object_shaders.object_states[object_id as usize],
            None => {
                error!("The state does not exist");
                return Err(EngineError::InvalidValue);
            }
        };

        // Release object descriptor sets
        let device = self.get_device()?;
        unsafe {
            if let Err(err) = device.free_descriptor_sets(
                object_shaders.per_object_descriptor_pool,
                &state.descriptor_sets,
            ) {
                error!(
                    "Failed to destroy descriptor sets of the current object: {:?}",
                    err
                );
                return Err(EngineError::ShutdownFailed);
            }
        }

        let object_shaders = &mut self
            .context
            .builtin_shaders
            .as_mut()
            .unwrap()
            .object_shaders;
        let state: &mut ObjectShadersPerObjectState =
            match object_shaders.object_states.get(object_id as usize) {
                Some(_) => &mut object_shaders.object_states[object_id as usize],
                None => {
                    error!("The state does not exist");
                    return Err(EngineError::InvalidValue);
                }
            };
        for i in 0..VULKAN_OBJECT_SHADERS_PER_OBJECT_DESCRIPTOR_COUNT {
            for j in 0..RENDERER_MAX_IN_FLIGHT_FRAMES {
                state.descriptor_states[i].generations[j] = None;
            }
        }
        Ok(())

        // TODO: add the object_id to the free list
    }
}
