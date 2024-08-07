use ash::vk::{self, PipelineLayout};

#[derive(Default)]
pub(crate) struct Pipeline{
    pub handler: vk::Pipeline,
    pub layout: PipelineLayout,
}