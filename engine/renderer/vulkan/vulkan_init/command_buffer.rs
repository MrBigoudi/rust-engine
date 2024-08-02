use ash::vk;

pub(crate) enum CommandBufferState {
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
    NotAllocated,
}

pub(crate) struct CommandBuffer {
    pub handler: vk::CommandBuffer,
    pub state: CommandBufferState,
}
