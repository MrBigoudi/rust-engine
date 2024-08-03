use ash::{
    vk::{self, FenceCreateFlags, FenceCreateInfo},
    Device,
};

use crate::{core::debug::errors::EngineError, error, warn};

#[derive(Clone, Copy)]
pub(crate) struct Fence {
    pub handler: vk::Fence,
    pub is_signaled: bool,
}

impl Fence {
    pub fn create(
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        is_signaled: bool,
    ) -> Result<Self, EngineError> {
        let mut fence_create_info = FenceCreateInfo::default();
        if is_signaled {
            fence_create_info.flags = FenceCreateFlags::SIGNALED
        };

        let handler = unsafe {
            match device.create_fence(&fence_create_info, allocator) {
                Ok(fence) => fence,
                Err(err) => {
                    error!("Failed to create a vulkan fence: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        Ok(Fence {
            handler,
            is_signaled,
        })
    }

    pub fn destroy(
        &mut self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        unsafe {
            device.destroy_fence(self.handler, allocator);
        };
        self.is_signaled = false;
        Ok(())
    }

    pub fn wait(
        &mut self,
        device: &Device,
        timeout_in_nanoseconds: u64,
    ) -> Result<bool, EngineError> {
        if self.is_signaled {
            return Ok(true);
        }

        let fences = [self.handler];
        unsafe {
            match device.wait_for_fences(&fences, true, timeout_in_nanoseconds) {
                Ok(()) => {
                    self.is_signaled = true;
                }
                Err(ash::vk::Result::TIMEOUT) => {
                    warn!(
                        "Warning waiting for a vulkan fence: {:?}",
                        ash::vk::Result::TIMEOUT
                    );
                }
                Err(err) => {
                    error!("Failed to wait for a vulkan fence: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        }

        Ok(false)
    }

    pub fn reset(&mut self, device: &Device) -> Result<(), EngineError> {
        if self.is_signaled {
            let fences = [self.handler];
            if let Err(err) = unsafe { device.reset_fences(&fences) } {
                error!("Failed to reset a vulkan fence: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
            self.is_signaled = false;
        }
        Ok(())
    }
}
