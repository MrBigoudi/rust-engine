use ash::{vk::{DeviceCreateInfo, DeviceQueueCreateInfo}, Device};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

use super::physical_device::PhysicalDeviceInfo;

impl VulkanRendererBackend<'_> {
    fn get_queue_create_infos(&self, physical_device_info: &PhysicalDeviceInfo) -> Result<Vec<DeviceQueueCreateInfo>, EngineError> {
        // NOTE: do not create additional queues for shared indices
        let present_shares_graphics_queue = physical_device_info.graphics_family_index == physical_device_info.present_family_index;
        let transfer_shares_graphics_queue = physical_device_info.graphics_family_index == physical_device_info.transfer_family_index;

        let mut queue_indices= vec![physical_device_info.graphics_family_index.unwrap()];
        if !present_shares_graphics_queue {
            queue_indices.push(physical_device_info.present_family_index.unwrap());
        }
        if !transfer_shares_graphics_queue {
            queue_indices.push(physical_device_info.transfer_family_index.unwrap());
        }
        
        let mut queue_create_infos: Vec<DeviceQueueCreateInfo> = Vec::new();
        for queue_index in queue_indices {
            let mut queue_create_info = DeviceQueueCreateInfo::default()
                .queue_family_index(queue_index as u32)
            ;
            // two queues for the graphics family, one for the other
            // TODO: change the queue priorities
            if queue_index == physical_device_info.graphics_family_index.unwrap() {
                queue_create_info = queue_create_info.queue_priorities(&[1., 1.]) 
            } else {
                queue_create_info = queue_create_info.queue_priorities(&[1.])
            }
            queue_create_infos.push(queue_create_info);
        } 

        Ok(queue_create_infos)
    }

    pub fn device_init(&mut self) -> Result<(), EngineError> {
        let physical_device_info = self.get_physical_device_info()?;

        let queue_create_infos = match self.get_queue_create_infos(physical_device_info) {
            Ok(infos) => infos,
            Err(err) => {
                error!("Failed to create the device queue infos: {:?}", err);
                return Err(EngineError::VulkanFailed)
            },
        };

        let device_features = physical_device_info.features;

        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_features(&physical_device_info.features)
        ;

        todo!()
    }

    pub fn device_shutdown(&mut self) -> Result<(), EngineError> {
        todo!()
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
