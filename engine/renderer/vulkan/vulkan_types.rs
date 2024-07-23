use ash::{
    ext::debug_utils,
    khr::surface,
    vk::{AllocationCallbacks, DebugUtilsMessengerEXT, PhysicalDevice, SurfaceKHR},
    Device, Entry, Instance,
};

use super::vulkan_init::device::PhysicalDeviceInfo;


#[derive(Default)]
pub(crate) struct VulkanContext<'a> {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub allocator: Option<&'a AllocationCallbacks<'a>>,

    pub debug_utils_loader: Option<debug_utils::Instance>,
    pub debug_callback: Option<DebugUtilsMessengerEXT>,

    pub surface_loader: Option<surface::Instance>,
    pub surface: Option<SurfaceKHR>,

    pub physical_device_info: Option<PhysicalDeviceInfo>,
    pub physical_device: Option<PhysicalDevice>,
    pub device: Option<Device>,
}

#[derive(Default)]
pub(crate) struct VulkanRendererBackend<'a> {
    pub context: VulkanContext<'a>,
    pub frame_number: u64,
}
