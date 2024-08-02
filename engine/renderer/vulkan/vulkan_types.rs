use ash::{
    ext::debug_utils,
    khr::surface,
    vk::{AllocationCallbacks, CommandPool, DebugUtilsMessengerEXT, PhysicalDevice, SurfaceKHR},
    Device, Entry, Instance,
};

use super::vulkan_init::{
    command_buffer::CommandBuffer,
    devices::{device_requirements::DeviceRequirements, physical_device::PhysicalDeviceInfo},
    renderpass::Renderpass,
    swapchain::Swapchain,
};

#[derive(Default)]
pub(crate) struct VulkanContext<'a> {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub allocator: Option<&'a AllocationCallbacks<'a>>,

    pub debug_utils_loader: Option<debug_utils::Instance>,
    pub debug_callback: Option<DebugUtilsMessengerEXT>,

    pub surface_loader: Option<surface::Instance>,
    pub surface: Option<SurfaceKHR>,

    pub device_requirements: Option<DeviceRequirements>,
    pub physical_device_info: Option<PhysicalDeviceInfo>,
    pub physical_device: Option<PhysicalDevice>,
    pub device: Option<Device>,

    pub swapchain: Option<Swapchain>,
    pub image_index: u32,
    pub current_frame: u32,
    pub should_recreate_swapchain: bool,

    pub renderpass: Option<Renderpass>,

    pub graphics_command_pool: Option<CommandPool>,
    pub graphics_command_buffers: Vec<CommandBuffer>,
}

#[derive(Default)]
pub(crate) struct VulkanRendererBackend<'a> {
    pub context: VulkanContext<'a>,

    pub frame_number: u64,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
}
