use ash::vk::{PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR};

pub(crate) struct SwapChainSupportDetails {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn is_complete(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}