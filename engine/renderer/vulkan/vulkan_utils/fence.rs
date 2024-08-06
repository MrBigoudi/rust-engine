use ash::{
    vk::{self, FenceCreateFlags, FenceCreateInfo},
    Device,
};

use crate::{core::debug::errors::EngineError, error, warn};

#[derive(Clone)]
pub(crate) struct Fence {
    pub handler: Box<vk::Fence>,
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
            handler: Box::new(handler),
        })
    }

    pub fn destroy(
        &self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        unsafe {
            device.destroy_fence(*self.handler.as_ref(), allocator);
        };
        Ok(())
    }

    pub fn wait(&self, device: &Device, timeout_in_nanoseconds: u64) -> Result<(), EngineError> {
        let fences = [*self.handler.as_ref()];
        unsafe {
            match device.wait_for_fences(&fences, true, timeout_in_nanoseconds) {
                Ok(()) => Ok(()),
                Err(ash::vk::Result::TIMEOUT) => {
                    warn!(
                        "Warning waiting for a vulkan fence: {:?}",
                        ash::vk::Result::TIMEOUT
                    );
                    Ok(())
                }
                Err(err) => {
                    error!("Failed to wait for a vulkan fence: {:?}", err);
                    Err(EngineError::VulkanFailed)
                }
            }
        }
    }

    pub fn reset(&self, device: &Device) -> Result<(), EngineError> {
        let fences = [*self.handler.as_ref()];
        if let Err(err) = unsafe { device.reset_fences(&fences) } {
            error!("Failed to reset a vulkan fence: {:?}", err);
            return Err(EngineError::VulkanFailed);
        }
        Ok(())
    }
}
