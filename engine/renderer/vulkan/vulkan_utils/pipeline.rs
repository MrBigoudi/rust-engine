use ash::{
    vk::{
        self, BlendFactor, BlendOp, ColorComponentFlags, CompareOp, CullModeFlags, DescriptorSetLayout, DynamicState, FrontFace, GraphicsPipelineCreateInfo, LogicOp, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, PushConstantRange, Rect2D, SampleCountFlags, ShaderStageFlags, VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, Viewport
    },
    Device,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::vulkan_init::{command_buffer::CommandBuffer, renderpass::Renderpass},
};

#[derive(Default)]
pub(crate) struct Pipeline {
    pub handler: vk::Pipeline,
    pub layout: PipelineLayout,
}

pub(crate) struct PipelineCreateInfo<'a> {
    pub renderpass: &'a Renderpass,
    pub viewports: Vec<Viewport>,
    pub scissors: Vec<Rect2D>,
    pub is_wireframe: bool,
    pub vertex_input_attributes_description: Vec<VertexInputAttributeDescription>,
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub shader_stages_info: Vec<PipelineShaderStageCreateInfo<'a>>,
}

impl Pipeline {
    pub fn create_graphics(
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        pipeline_info: PipelineCreateInfo,
    ) -> Result<Self, EngineError> {
        // Viewport state
        let viewport_create_info = PipelineViewportStateCreateInfo::default()
            .viewports(&pipeline_info.viewports)
            .scissors(&pipeline_info.scissors);

        // Rasterizer
        let rasterizer_create_info = PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(if pipeline_info.is_wireframe {
                PolygonMode::LINE
            } else {
                PolygonMode::FILL
            })
            .line_width(1.0)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::COUNTER_CLOCKWISE);

        // Multisampling
        let multisampling_create_info = PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0);

        // Depth and stencil
        let depth_stencil_create_info = PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(CompareOp::LESS);

        // Color blending
        let color_blend_attachment_states = [PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(BlendOp::ADD)
            .src_alpha_blend_factor(BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(BlendOp::ADD)
            .color_write_mask(ColorComponentFlags::RGBA)];
        let color_blend_create_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op(LogicOp::COPY)
            .attachments(&color_blend_attachment_states);

        // Dynamic state
        let dynamic_states = vec![
            DynamicState::VIEWPORT,
            DynamicState::SCISSOR,
            DynamicState::LINE_WIDTH,
        ];
        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Vertex Input
        let vertex_input_binding_descriptions = [
            VertexInputBindingDescription::default()
                // vec3 position at binding = 0
                .binding(0)
                .stride(size_of::<glam::Vec3>() as u32)
                .input_rate(VertexInputRate::VERTEX), // move to next data entry for each vertex
        ];
        let vertex_input_create_info = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_input_binding_descriptions)
            .vertex_attribute_descriptions(&pipeline_info.vertex_input_attributes_description);

        // Input assembly
        let input_assembly_create_info = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST);

        // Push constants
        let push_constant_ranges = [
            PushConstantRange::default()
                .stage_flags(ShaderStageFlags::VERTEX) // only push constants to vertex shader
                .offset(0)
                .size((size_of::<glam::Mat4>()) as u32), // max size of 128 bytes
        ];

        // Pipeline layout
        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default()
            .set_layouts(&pipeline_info.descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout = unsafe {
            match device.create_pipeline_layout(&pipeline_layout_create_info, allocator) {
                Ok(layout) => layout,
                Err(err) => {
                    error!(
                        "Failed to create a vulkan pipeline layout in a graphics pipeline: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        // Pipeline
        let graphics_pipeline_create_info = [GraphicsPipelineCreateInfo::default()
            .stages(&pipeline_info.shader_stages_info)
            .vertex_input_state(&vertex_input_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisampling_create_info)
            .depth_stencil_state(&depth_stencil_create_info)
            .color_blend_state(&color_blend_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(pipeline_layout)
            .render_pass(pipeline_info.renderpass.handler)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
        ];

        let pipeline = unsafe {
            match device.create_graphics_pipelines(
                PipelineCache::null(),
                &graphics_pipeline_create_info,
                allocator,
            ) {
                Ok(pipelines) => pipelines[0],
                Err(err) => {
                    error!(
                        "Failed to create vulkan pipelines in a graphics pipeline: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        Ok(Self {
            handler: pipeline,
            layout: pipeline_layout,
        })
    }

    pub fn destroy(
        &self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        unsafe {
            device.destroy_pipeline(self.handler, allocator);
            device.destroy_pipeline_layout(self.layout, allocator);
        }

        Ok(())
    }

    pub fn bind(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        bind_point: PipelineBindPoint,
    ) -> Result<(), EngineError> {
        unsafe {
            device.cmd_bind_pipeline(*command_buffer.handler.as_ref(), bind_point, self.handler);
        }
        Ok(())
    }
}
