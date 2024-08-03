use std::cmp::{max, min};

use ash::{
    vk::{self, FramebufferCreateInfo, ImageView},
    Device,
};

use crate::{
    core::{application::application_get_framebuffer_size, debug::errors::EngineError},
    error,
    renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

use super::renderpass::Renderpass;

#[derive(Clone)]
pub(crate) struct Framebuffer {
    pub handler: vk::Framebuffer,
    pub attachments: Vec<ImageView>,
    pub renderpass: Renderpass,
}

impl Framebuffer {
    pub fn create(
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        renderpass: &Renderpass,
        width: u32,
        height: u32,
        attachments: &[ImageView],
    ) -> Result<Self, EngineError> {
        // Take a copy of the attachments, renderpass and attachment count
        let renderpass = *renderpass;
        let attachments = attachments.to_owned();

        let framebuffer_info = FramebufferCreateInfo::default()
            .render_pass(renderpass.handler)
            .attachments(&attachments)
            .width(width)
            .height(height)
            .layers(1);

        let handler = unsafe {
            match device.create_framebuffer(&framebuffer_info, allocator) {
                Ok(framebuffer) => framebuffer,
                Err(err) => {
                    error!("Failed to create a vulkan framebuffer: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        Ok(Framebuffer {
            handler,
            attachments,
            renderpass,
        })
    }

    pub fn destroy(
        &mut self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        unsafe {
            device.destroy_framebuffer(self.handler, allocator);
        }
        self.attachments.clear();
        Ok(())
    }
}

impl VulkanRendererBackend<'_> {
    pub fn framebuffer_dimensions_init(&mut self) -> Result<(), EngineError> {
        let (width, height) = application_get_framebuffer_size()?;
        // TODO: find other solution for framebuffer size
        // Clamp framebuffer to swapchain surface capacity
        let swapchain_support_max_extent = self.get_swapchain_support_details()?.capabilities.max_image_extent;
        let swapchain_support_min_extent = self.get_swapchain_support_details()?.capabilities.min_image_extent;
        self.framebuffer_width = min(swapchain_support_max_extent.width, max(swapchain_support_min_extent.width, width));
        self.framebuffer_height = min(swapchain_support_max_extent.height, max(swapchain_support_min_extent.height, height));
        Ok(())
    }

    pub fn swapchain_framebuffers_shutdown(&mut self) -> Result<(), EngineError> {
        let framebuffers = &self.context.swapchain.as_mut().unwrap().framebuffers;
        for mut buffer in framebuffers.clone() {
            buffer.destroy(self.get_device()?, self.get_allocator()?)?;
        }
        Ok(())
    }

    pub fn swapchain_framebuffers_init(&mut self) -> Result<(), EngineError> {
        // destroy swapchain framebuffers
        self.swapchain_framebuffers_shutdown()?;

        let depth_attachment = self.get_swapchain()?.depth_attachment.as_ref().unwrap();
        let image_views: &Vec<ImageView> = self.get_swapchain()?.image_views.as_ref();

        let mut framebuffers = Vec::new();

        for image_view in image_views {
            // TODO: make this dynamic based on the currently configured attachments
            let attachments = vec![
                *image_view,
                depth_attachment.image_view.unwrap(),
            ];
            let new_framebuffer = Framebuffer::create(
                self.get_device()?,
                self.get_allocator()?,
                self.get_renderpass()?,
                self.framebuffer_width,
                self.framebuffer_height,
                &attachments,
            )?;
            framebuffers.push(new_framebuffer);
        }

        self.context.swapchain.as_mut().unwrap().framebuffers = framebuffers.clone();

        Ok(())
    }
}
