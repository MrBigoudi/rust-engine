use ash::{
    vk::{
        self, Extent2D, Format, Offset2D, PipelineShaderStageCreateInfo, Rect2D, ShaderStageFlags,
        VertexInputAttributeDescription, Viewport,
    },
    Device,
};

use crate::{
    core::debug::errors::EngineError,
    renderer::vulkan::{
        vulkan_shaders::shader::Shader,
        vulkan_types::VulkanRendererBackend,
        vulkan_utils::pipeline::{Pipeline, PipelineCreateInfo},
    },
};

/// Default shader to display objects
pub(crate) struct ObjectShaders {
    pub vertex_stage: Shader,
    pub fragment_stage: Shader,
    pub pipeline: Pipeline,
}

impl ObjectShaders {
    fn create_pipeline_info<'a>(
        backend: &'a VulkanRendererBackend<'a>,
        vertex_shader: &'a Shader,
        fragment_shader: &'a Shader,
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

        // TODO: Desciptor set layouts.

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
            descriptor_set_layouts: Vec::new(),
            shader_stages_info,
        })
    }

    pub fn create(backend: &VulkanRendererBackend<'_>) -> Result<Self, EngineError> {
        let device = backend.get_device()?;
        let allocator = backend.get_allocator()?;

        // Shader module init per stage
        let vertex_stage = Shader::create(
            device,
            allocator,
            ShaderStageFlags::VERTEX,
            "builtin/object.vert.slang",
            None,
        )?;

        let fragment_stage = Shader::create(
            device,
            allocator,
            ShaderStageFlags::FRAGMENT,
            "builtin/object.frag.slang",
            None,
        )?;

        // Descriptors
        // TODO: create a pipeline
        let pipeline_info = Self::create_pipeline_info(backend, &vertex_stage, &fragment_stage)?;
        let pipeline = Pipeline::create_graphics(device, allocator, pipeline_info)?;

        Ok(ObjectShaders {
            vertex_stage,
            fragment_stage,
            pipeline,
        })
    }

    pub fn destroy(
        &self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        self.pipeline.destroy(device, allocator)?;
        self.vertex_stage.destroy(device, allocator)?;
        self.fragment_stage.destroy(device, allocator)?;
        Ok(())
    }
}
