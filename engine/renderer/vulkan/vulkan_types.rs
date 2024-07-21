use ash::{
    ext::debug_utils, vk::{AllocationCallbacks, DebugUtilsMessengerEXT, PhysicalDevice}, Device, Entry, Instance
};

#[derive(Default)]
pub(crate) struct VulkanContext<'a> {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub allocator: Option<&'a AllocationCallbacks<'a>>,
    
    pub debug_utils_loader: Option<debug_utils::Instance>,
    pub debug_callback: Option<DebugUtilsMessengerEXT>,

    pub physical_device: Option<PhysicalDevice>,
    pub device: Option<Device>,
}

#[derive(Default)]
pub(crate) struct VulkanRendererBackend<'a> {
    pub context: VulkanContext<'a>,
    pub frame_number: u64,
}
