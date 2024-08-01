use ash::vk::{PhysicalDevice, Queue, QueueFlags};

use crate::{
    core::debug::errors::EngineError, error, renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

#[derive(Debug, Default)]
pub(crate) struct Queues {
    pub graphics_family_index: Option<usize>,
    pub graphics_family_queue_count: Option<u32>,
    pub graphics_queue: Option<Queue>,

    pub present_family_index: Option<usize>,
    pub present_family_queue_count: Option<u32>,
    pub present_queue: Option<Queue>,

    pub compute_family_index: Option<usize>,
    pub compute_family_queue_count: Option<u32>,
    pub compute_queue: Option<Queue>,

    pub transfer_family_index: Option<usize>,
    pub transfer_family_queue_count: Option<u32>,
    pub transfer_queue: Option<Queue>,
}

impl VulkanRendererBackend<'_> {
    pub(crate) fn queue_family_properties_create(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<Queues, EngineError> {
        let queue_family_properties = unsafe {
            self.get_instance()?
                .get_physical_device_queue_family_properties(*physical_device)
        };

        let mut queues = Queues::default();

        let mut min_transfer_score = u32::MAX;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            let mut transfer_score = 0;

            // Graphics queue ?
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                queues.graphics_family_index = Some(index);
                queues.graphics_family_queue_count = Some(queue_family.queue_count);
                transfer_score += 1;
            }

            // Compute queue ?
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE) {
                queues.compute_family_index = Some(index);
                queues.compute_family_queue_count = Some(queue_family.queue_count);
                transfer_score += 1;
            }

            // Transfer queue ?
            if queue_family.queue_flags.contains(QueueFlags::TRANSFER) {
                // Take the index if it is the current lowest. This increases the
                // likelihood that it is a dedicated transfer queue.
                if transfer_score <= min_transfer_score {
                    min_transfer_score = transfer_score;
                    queues.transfer_family_index = Some(index);
                    queues.transfer_family_queue_count = Some(queue_family.queue_count);
                }
            }

            // Present queue ?
            match unsafe {
                self.get_surface_loader()?
                    .get_physical_device_surface_support(
                        *physical_device,
                        index as u32,
                        *self.get_surface()?,
                    )
            } {
                Ok(false) => (),
                Ok(true) => {
                    queues.present_family_index = Some(index);
                    queues.present_family_queue_count = Some(queue_family.queue_count);
                }
                Err(err) => {
                    error!(
                        "Failed to fetch the physical device surface support: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        }
        Ok(queues)
    }

    pub fn queues_init(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        let graphics_queue;
        let present_queue;
        let compute_queue;
        let transfer_queue;

        unsafe {
            graphics_queue = device
                .get_device_queue(self.get_queues()?.graphics_family_index.unwrap() as u32, 0);
            present_queue =
                device.get_device_queue(self.get_queues()?.present_family_index.unwrap() as u32, 0);
            compute_queue =
                device.get_device_queue(self.get_queues()?.compute_family_index.unwrap() as u32, 0);
            transfer_queue = device
                .get_device_queue(self.get_queues()?.transfer_family_index.unwrap() as u32, 0);
        }

        let physical_device_info = self.context.physical_device_info.as_mut().unwrap();
        physical_device_info.queues.graphics_queue = Some(graphics_queue);
        physical_device_info.queues.present_queue = Some(present_queue);
        physical_device_info.queues.compute_queue = Some(compute_queue);
        physical_device_info.queues.transfer_queue = Some(transfer_queue);

        Ok(())
    }

    pub fn get_queues(&self) -> Result<&Queues, EngineError> {
        match self.get_physical_device_info() {
            Ok(info) => Ok(&info.queues),
            Err(err) => {
                error!("Can't access the vulkan queues: {:?}", err);
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn queues_shutdown(&mut self) -> Result<(), EngineError> {
        Ok(())
    }
}
