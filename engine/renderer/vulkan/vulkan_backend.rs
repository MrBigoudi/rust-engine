use ash::vk::{Extent2D, Fence, PipelineStageFlags, Rect2D, SubmitInfo, Viewport};

use crate::{
    core::debug::errors::EngineError, error, platforms::platform::Platform,
    renderer::renderer_backend::RendererBackend,
};

use super::{vulkan_types::VulkanRendererBackend, vulkan_utils::texture::Texture};

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
            if let Err(err) = self.swapchain_recreate() {
                error!(
                    "Failed to recreate the vulkan swapchain when beginning a new frame: {:?}",
                    err
                );
                return Err(EngineError::Unknown);
            }
            self.context.has_framebuffer_been_resized = false;
            return Ok(false);
        }

        // Wait for the execution of the current frame to complete. The fence being free will allow this one to move on
        let current_frame_index = self.context.current_frame as usize;
        let current_image_fence =
            &self.get_sync_structures()?.in_flight_fences[current_frame_index];
        let device = self.get_device()?;
        let timeout = u64::MAX;
        if let Err(err) = current_image_fence.wait(device, timeout) {
            error!(
                "Failed to wait for the current image fence when beginning a new frame: {:?}",
                err
            );
            return Err(EngineError::Unknown);
        }

        // Acquire the next image from the swap chain. Pass along the semaphore that should signaled when this completes
        // This same semaphore will later be waited on by the queue submission to ensure this image is available
        let image_available_semaphore =
            self.get_sync_structures()?.image_available_semaphores[current_frame_index];

        let next_image_index =
            self.get_swapchain_next_image_index(timeout, image_available_semaphore, Fence::null())?;
        if let Some(index) = next_image_index {
            self.context.image_index = index;
        } else {
            if let Err(err) = self.swapchain_recreate() {
                error!("Failed to recreate the vulkan swapchain when acquiring a wrong image at the beginning of a new frame: {:?}", err);
                return Err(EngineError::InitializationFailed);
            }
            return Ok(false);
        }
        let current_image_fence =
            &self.get_sync_structures()?.in_flight_fences[current_frame_index];
        let device = self.get_device()?;
        if let Err(err) = current_image_fence.reset(device) {
            error!(
                "Failed to reset the current image fence when beginning a new frame: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        // Begin recording commands
        let command_buffer = &self.context.graphics_command_buffers[current_frame_index];
        let device = self.get_device()?;
        if let Err(err) = command_buffer.reset(device) {
            error!(
                "Failed to reset the current command buffer when beginning a new frame: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }
        if let Err(err) = command_buffer.begin(device, false, false, false) {
            error!(
                "Failed to begin the current command buffer when beginning a new frame: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

        // Begin the render pass
        let image_index = self.context.image_index as usize;
        let framebuffer = &self.get_swapchain()?.framebuffers[image_index];
        if let Err(err) = self.renderpass_begin(command_buffer, *framebuffer.handler.as_ref()) {
            error!(
                "Failed to begin the renderpass when beginning a new frame: {:?}",
                err
            );
            return Err(EngineError::InitializationFailed);
        }

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
        if let Err(err) = self.renderpass_end(command_buffer) {
            error!(
                "Failed to end the renderpass when ending a new frame: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }
        let device = self.get_device()?;
        if let Err(err) = command_buffer.end(device) {
            error!(
                "Failed to end the current command buffer when ending a new frame: {:?}",
                err
            );
            return Err(EngineError::ShutdownFailed);
        }

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
                error!(
                    "Failed to submit the vulkan graphics queue when ending a new frame: {:?}",
                    err
                );
                return Err(EngineError::VulkanFailed);
            }
        }

        // Give the image back to the swapchain.
        let render_complete_semaphore =
            self.get_sync_structures()?.queue_complete_semaphores[current_frame_index];
        match self.swapchain_present(render_complete_semaphore, self.context.image_index) {
            Ok(Some(())) => (),
            Ok(None) => self.swapchain_recreate()?,
            Err(err) => {
                error!(
                    "Failed to present the vulkan swapchain when ending a new frame: {:?}",
                    err
                );
                return Err(err);
            }
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

    fn update_global_state(
        &mut self,
        projection: glam::Mat4,
        view: glam::Mat4,
        view_position: glam::Vec3,
        ambient_colour: glam::Vec4,
        mode: i32,
    ) -> Result<(), EngineError> {
        let current_frame_index = self.context.current_frame as usize;
        let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
        let device = self.get_device()?;

        let object_shaders = &self.get_builtin_shaders()?.object_shaders;
        object_shaders.r#use(device, command_buffer)?;
        let object_shaders = &mut self
            .context
            .builtin_shaders
            .as_mut()
            .unwrap()
            .object_shaders;
        object_shaders.global_ubo.projection = projection;
        object_shaders.global_ubo.view = view;

        // TODO: other ubo properties
        if let Err(err) = self.update_object_shaders_global_state() {
            error!(
                "Failed to update the vulkan object shaders global state: {:?}",
                err
            );
            return Err(EngineError::UpdateFailed);
        }

        Ok(())
    }

    fn get_aspect_ratio(&self) -> Result<f32, EngineError> {
        let width = self.get_swapchain()?.extent.width as f32;
        let height = self.get_swapchain()?.extent.width as f32;
        Ok(width / height)
    }

    fn update_object(&mut self, model: glam::Mat4) -> Result<(), EngineError> {
        let current_frame_index = self.context.current_frame as usize;
        if let Err(err) = self.update_object_shaders(model) {
            error!(
                "Failed to update the vulkan object shaders when updating the vulkan objects: {:?}",
                err
            );
            return Err(EngineError::UpdateFailed);
        }

        // TODO: temporary test code
        {
            let object_shaders = &self.get_builtin_shaders()?.object_shaders;
            let image_index = self.context.image_index as usize;
            let command_buffer = &self.get_graphics_command_buffers()?[current_frame_index];
            let device = self.get_device()?;
            object_shaders.r#use(device, command_buffer)?;
            // Bind vertex buffer at offset
            let offsets = [0];
            let vertex_buffer = [self.get_objects_buffers()?.vertex_buffer.buffer];
            unsafe {
                device.cmd_bind_vertex_buffers(
                    *command_buffer.handler.as_ref(),
                    0,
                    &vertex_buffer,
                    &offsets,
                );
            }
            // Bind index buffer at offset
            let index_buffer = self.get_objects_buffers()?.index_buffer.buffer;
            unsafe {
                device.cmd_bind_index_buffer(
                    *command_buffer.handler.as_ref(),
                    index_buffer,
                    0,
                    ash::vk::IndexType::UINT32,
                );
            }
            // Issue the draw
            unsafe {
                device.cmd_draw_indexed(*command_buffer.handler.as_ref(), 6, 1, 0, 0, 0);
            }
        }
        // TODO: end temporary test code
        Ok(())
    }

    fn create_texture(
        &self,
        params: crate::resources::texture::TextureCreatorParameters,
    ) -> Result<Box<dyn crate::resources::texture::Texture>, EngineError> {
        let vulkan_texture = match self.vulkan_create_texture(params) {
            Ok(texture) => texture,
            Err(err) => {
                error!(
                    "Failed to create a vulkan texture when creating a renderer texture: {:?}",
                    err
                );
                return Err(EngineError::InitializationFailed);
            }
        };
        Ok(Box::new(vulkan_texture))
    }

    fn destroy_texture(
        &self,
        texture: Box<dyn crate::resources::texture::Texture>,
    ) -> Result<(), EngineError> {
        let vulkan_texture = match texture.as_any().downcast_ref::<Texture>() {
            Some(texture) => texture,
            None => {
                error!("A vulkan renderer can only destroy vulkan textures");
                return Err(EngineError::InvalidValue);
            }
        };
        if let Err(err) = self.vulkan_destroy_texture(vulkan_texture) {
            error!("Failed to destroy a vulkan texture: {:?}", err);
            return Err(EngineError::ShutdownFailed);
        }
        Ok(())
    }
}
