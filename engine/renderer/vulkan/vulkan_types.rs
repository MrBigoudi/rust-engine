use ash::{vk::AllocationCallbacks, Entry, Instance};

#[derive(Default)]
pub(crate) struct VulkanContext<'a> {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub allocator: Option<&'a AllocationCallbacks<'a>>,
}

#[derive(Default)]
pub(crate) struct VulkanRendererBackend<'a> {
    pub context: VulkanContext<'a>,
    pub frame_number: u64,
}
