use ash::vk::{Semaphore, SemaphoreCreateInfo};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::vulkan::{vulkan_types::VulkanRendererBackend, vulkan_utils::fence::Fence},
};

pub(crate) struct SyncStructure {
    pub image_available_semaphores: Vec<Semaphore>,
    pub queue_complete_semaphores: Vec<Semaphore>,
    pub in_flight_fences: Vec<Fence>,
}

impl VulkanRendererBackend<'_> {
    pub fn get_sync_structures(&self) -> Result<&SyncStructure, EngineError> {
        match &self.context.sync_structures {
            Some(sync_structures) => Ok(sync_structures),
            None => {
                error!("Can't access the vulkan sync structures");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn sync_structures_init(&mut self) -> Result<(), EngineError> {
        let mut image_available_semaphores = Vec::new();
        let mut queue_complete_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();

        // Create sync objects
        let max_frames_in_flight = self.get_swapchain()?.max_frames_in_flight;
        let device = self.get_device()?;
        let allocator = self.get_allocator()?;
        let semaphore_info = SemaphoreCreateInfo::default();
        for _ in 0..max_frames_in_flight {
            image_available_semaphores.push(self.create_default_semaphore()?);
            queue_complete_semaphores.push(self.create_default_semaphore()?);
            in_flight_fences.push(Fence::create(device, allocator, true)?);
        }

        self.context.sync_structures = Some(SyncStructure {
            image_available_semaphores,
            queue_complete_semaphores,
            in_flight_fences,
        });

        Ok(())
    }

    pub fn sync_structures_shutdown(&mut self) -> Result<(), EngineError> {
        let max_frames_in_flight = self.get_swapchain()?.max_frames_in_flight as usize;

        // destroy semaphores
        for index in 0..max_frames_in_flight {
            self.destroy_semaphore(&self.get_sync_structures()?.image_available_semaphores[index])?;
            self.destroy_semaphore(&self.get_sync_structures()?.queue_complete_semaphores[index])?;
        }

        // destroy fences
        let in_flight_fences = &self
            .context
            .sync_structures
            .as_ref()
            .unwrap()
            .in_flight_fences;
        for fence in in_flight_fences {
            fence.destroy(self.get_device()?, self.get_allocator()?)?;
        }

        // empty vectors
        let sync_structures = self.context.sync_structures.as_mut().unwrap();
        sync_structures.image_available_semaphores.clear();
        sync_structures.queue_complete_semaphores.clear();
        sync_structures.in_flight_fences.clear();

        Ok(())
    }
}
