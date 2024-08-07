use ash::{vk::ShaderStageFlags, Device};

use crate::{core::debug::errors::EngineError, renderer::vulkan::{vulkan_pipelines::pipeline::Pipeline, vulkan_shaders::shader::Shader}};

/// Default shader to display objects
pub(crate) struct ObjectShaders {
    pub vertex_stage: Shader,
    pub fragment_stage: Shader,
    pub pipeline: Pipeline,
}

impl ObjectShaders {
    pub fn create(device: &Device) -> Result<Self, EngineError> {
        // Shader module init per stage
        let vertex_stage = Shader::create(
            device, 
            ShaderStageFlags::VERTEX, 
            "builtin_object.vert.slang",
            None 
        )?;

        let fragment_stage = Shader::create(
            device, 
            ShaderStageFlags::VERTEX, 
            "builtin_object.frag.slang",
            None 
        )?;

        // Descriptors
        // TODO: create a pipeline

        Ok(ObjectShaders{
            vertex_stage,
            fragment_stage,
            pipeline: Pipeline::default(),
        })
    }
}
