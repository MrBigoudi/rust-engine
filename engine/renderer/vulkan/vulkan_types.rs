use ash::{vk::AllocationCallbacks, Entry, Instance};

use crate::{core::errors::EngineError, error};

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

impl VulkanRendererBackend<'_> {
    pub fn get_entry(&self) -> Result<&Entry, EngineError> {
        match &self.context.entry {
            Some(entry) => Ok(entry),
            None => {
                error!("Can't access the vulkan entry");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn get_instance(&self) -> Result<&Instance, EngineError> {
        match &self.context.instance {
            Some(instance) => Ok(instance),
            None => {
                error!("Can't access the vulkan instance");
                Err(EngineError::AccessFailed)
            }
        }
    }
}

impl<'a> VulkanRendererBackend<'a> {
    pub fn get_allocator(&self) -> Result<Option<&'a AllocationCallbacks<'a>>, EngineError> {
        Ok(self.context.allocator)
    }
}
