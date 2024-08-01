use ash::{vk::{DeviceCreateInfo, DeviceQueueCreateInfo}, Device};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    fn get_device_queue_create_infos(&self) -> Result<Vec<DeviceQueueCreateInfo>, EngineError> {
        // NOTE: do not create additional queues for shared indices
        let present_shares_graphics_queue = self.get_queues()?.graphics_family_index == self.get_queues()?.present_family_index;
        let transfer_shares_graphics_queue = self.get_queues()?.graphics_family_index == self.get_queues()?.transfer_family_index;

        let mut queue_indices= vec![self.get_queues()?.graphics_family_index.unwrap()];
        if !present_shares_graphics_queue {
            queue_indices.push(self.get_queues()?.present_family_index.unwrap());
        }
        if !transfer_shares_graphics_queue {
            queue_indices.push(self.get_queues()?.transfer_family_index.unwrap());
        }
        
        let mut queue_create_infos: Vec<DeviceQueueCreateInfo> = Vec::new();
        for queue_index in queue_indices {
            let queue_create_info = DeviceQueueCreateInfo::default()
                .queue_family_index(queue_index as u32)
                // TODO: change the queue priorities
                .queue_priorities(&[1.])
            ;
            queue_create_infos.push(queue_create_info);
        } 

        Ok(queue_create_infos)
    }


    pub fn device_init(&mut self) -> Result<(), EngineError> {
        let physical_device_info = self.get_physical_device_info()?;

        let queue_create_infos = match self.get_device_queue_create_infos() {
            Ok(infos) => infos,
            Err(err) => {
                error!("Failed to create the device queue infos: {:?}", err);
                return Err(EngineError::VulkanFailed)
            },
        };

        let requirements = self.get_device_requirements()?;

        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_features(&requirements.features)
            .enabled_extension_names(requirements.extensions.as_slice())
        ;

        unsafe {
            match
            self.get_instance()?
                .create_device(
                    *self.get_physical_device()?, 
                    &device_create_info, 
                    self.get_allocator()?
                ) {
                Ok(device) => self.context.device = Some(device),
                Err(err) => {
                    error!("Failed to initialize the vulkan logical device: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                } 
            }
        }

        Ok(())
    }

    pub fn device_shutdown(&mut self) -> Result<(), EngineError> {
        unsafe {
            self.get_device()?.destroy_device(self.get_allocator()?);
        }
        self.context.device = None;
        Ok(())
    }

    pub fn get_device(&self) -> Result<&Device, EngineError> {
        match &self.context.device {
            Some(device) => Ok(device),
            None => {
                error!("Can't access the vulkan device");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
