use ash::vk::{Extent2D, Fence, PipelineStageFlags, Rect2D, SubmitInfo, Viewport};

use crate::{
    core::debug::errors::EngineError, error, platforms::platform::Platform,
    renderer::renderer_backend::RendererBackend,
};

use super::vulkan_types::VulkanRendererBackend;

impl RendererBackend for VulkanRendererBackend<'_> {
    fn init(&mut self, application_name: &str, platform: &dyn Platform) -> Result<(), EngineError> {
        self.vulkan_init(application_name, platform)?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), EngineError> {
        self.vulkan_shutdown()?;
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), EngineError> {
        self.swapchain_recreate()?;
        Ok(())
    }

    fn begin_frame(&mut self, delta_time: f64) -> Result<bool, EngineError> {
        if self.context.has_framebuffer_been_resized {
            self.swapchain_recreate()?;
            self.context.has_framebuffer_been_resized = false;
            return Ok(false);
        }

        // Wait for the execution of the current frame to complete. The fence being free will allow this one to move on
        let current_frame_index = self.context.current_frame as usize;
        let current_image_fence =
            &self.get_sync_structures()?.in_flight_fences[current_frame_index];
        let device = self.get_device()?;
        let timeout = u64::MAX;
        current_image_fence.wait(device, timeout)?;

        // Acquire the next image from the swap chain. Pass along the semaphore that should signaled when this completes
        // This same semaphore will later be waited on by the queue submission to ensure this image is available
        let image_available_semaphore =
            self.get_sync_structures()?.image_available_semaphores[current_frame_index];

        let next_image_index =
            self.get_swapchain_next_image_index(timeout, image_available_semaphore, Fence::null())?;
        if let Some(index) = next_image_index {
            self.context.image_index = index;
        } else {
            self.swapchain_recreate()?;
            return Ok(false);
        }
        let current_image_fence =
            &self.get_sync_structures()?.in_flight_fences[current_frame_index];
        let device = self.get_device()?;
        current_image_fence.reset(device)?;

        // Begin recording commands
        let command_buffer = &self.context.graphics_command_buffers[current_frame_index];
        let device = self.get_device()?;
        command_buffer.reset(device)?;
        command_buffer.begin(device, false, false, false)?;

        // Begin the render pass
        let image_index = self.context.image_index as usize;
        let framebuffer = &self.get_swapchain()?.framebuffers[image_index];
        self.renderpass_begin(command_buffer, *framebuffer.handler.as_ref())?;

        // Dynamic viewport
        let render_area = self.get_renderpass()?.render_area;
        let viewport = [Viewport::default()
            .x(0.)
            .y(render_area.height)
            .width(render_area.width)
            .height(-render_area.height)
            .min_depth(0.)
            .max_depth(1.)];
        unsafe { device.cmd_set_viewport(*command_buffer.handler.as_ref(), 0, &viewport) };

        // Dynamic scissor
        let scissor = [Rect2D::default().extent(Extent2D {
            width: self.framebuffer_width,
            height: self.framebuffer_height,
        })];
        let device = self.get_device()?;
        unsafe { device.cmd_set_scissor(*command_buffer.handler.as_ref(), 0, &scissor) };

        Ok(true)
    }

    fn end_frame(&mut self, delta_time: f64) -> Result<(), EngineError> {
        let current_frame_index = self.context.current_frame as usize;

        // End renderpass
        let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
        self.renderpass_end(command_buffer)?;
        let device = self.get_device()?;
        command_buffer.end(device)?;

        // Submit the queue and wait for the operation to complete
        let command_buffers = [*command_buffer.handler.as_ref()];
        let signal_semaphores =
            [self.get_sync_structures()?.queue_complete_semaphores[current_frame_index]];
        let wait_semaphores =
            [self.get_sync_structures()?.image_available_semaphores[current_frame_index]];
        let wait_dst_stage_mask = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = [SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_dst_stage_mask)];
        let current_fence = &self.get_sync_structures()?.in_flight_fences[current_frame_index];
        let graphics_queue = self.get_queues()?.graphics_queue.unwrap();
        let device = self.get_device()?;
        unsafe {
            if let Err(err) = device.queue_submit(
                graphics_queue,
                &submit_info,
                *current_fence.handler.as_ref(),
            ) {
                error!("Failed to submit the vulkan graphics queue: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        }

        // Give the image back to the swapchain.
        let render_complete_semaphore =
            self.get_sync_structures()?.queue_complete_semaphores[current_frame_index];
        match self.swapchain_present(render_complete_semaphore, self.context.image_index) {
            Ok(Some(())) => (),
            Ok(None) => self.swapchain_recreate()?,
            Err(err) => return Err(err),
        }

        Ok(())
    }

    fn increase_frame_number(&mut self) -> Result<(), EngineError> {
        self.frame_number += 1;
        Ok(())
    }

    fn get_frame_number(&self) -> Result<u64, EngineError> {
        Ok(self.frame_number)
    }
}
