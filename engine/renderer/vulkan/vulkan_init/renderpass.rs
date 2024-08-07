use ash::vk::{
    self, AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference,
    AttachmentStoreOp, ClearColorValue, ClearDepthStencilValue, ClearValue, Extent2D, Framebuffer,
    ImageLayout, Offset2D, PipelineBindPoint, PipelineStageFlags, Rect2D, RenderPassBeginInfo,
    RenderPassCreateInfo, SampleCountFlags, SubpassContents, SubpassDependency, SubpassDescription,
    SUBPASS_EXTERNAL,
};

use crate::{
    core::debug::errors::EngineError,
    error,
    renderer::{
        utils::{color::Color, render_area::RenderArea},
        vulkan::vulkan_types::VulkanRendererBackend,
    },
};

use super::command_buffer::CommandBuffer;

pub(crate) enum RenderpassState {
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
    NotAllocated,
}

pub(crate) struct Renderpass {
    pub handler: vk::RenderPass,
    pub render_area: RenderArea,
    pub clear_color: Color,
    pub depth: f32,
    pub stencil: u32,
    pub state: RenderpassState,
}

impl VulkanRendererBackend<'_> {
    fn init_color_attachment(&self) -> Result<AttachmentDescription, EngineError> {
        // TODO: make the renderpass attachments configurable
        let format = self.get_swapchain()?.surface_format.format;
        Ok(
            AttachmentDescription::default()
                .format(format)
                .samples(SampleCountFlags::TYPE_1)
                .load_op(AttachmentLoadOp::CLEAR)
                .store_op(AttachmentStoreOp::STORE)
                .stencil_load_op(AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(AttachmentStoreOp::DONT_CARE)
                .initial_layout(ImageLayout::UNDEFINED) // Do not expect any particular layout before render pass starts
                .final_layout(ImageLayout::PRESENT_SRC_KHR), // Transitioned to after the render pass
        )
    }

    fn init_depth_attachment(&self) -> Result<Option<AttachmentDescription>, EngineError> {
        // TODO: make the renderpass attachments configurable
        let format = self.get_physical_device_info()?.depth_format;
        if let Some(format) = format {
            Ok(Some(
                AttachmentDescription::default()
                    .format(format)
                    .samples(SampleCountFlags::TYPE_1)
                    .load_op(AttachmentLoadOp::CLEAR)
                    .store_op(AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(AttachmentStoreOp::DONT_CARE)
                    .initial_layout(ImageLayout::UNDEFINED)
                    .final_layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ))
        } else {
            Ok(None)
        }
    }

    fn init_dependencies(&self) -> Result<SubpassDependency, EngineError> {
        // TODO: make the renderpass dependencies configurable
        Ok(SubpassDependency::default()
            .src_subpass(SUBPASS_EXTERNAL)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                AccessFlags::COLOR_ATTACHMENT_READ | AccessFlags::COLOR_ATTACHMENT_WRITE,
            ))
    }

    pub fn renderpass_render_area_clamp(&mut self) -> Result<(), EngineError> {
        self.framebuffer_dimensions_init()?;
        let width = self.framebuffer_width as f32;
        let height = self.framebuffer_height as f32;
        let render_area = &mut self.context.renderpass.as_mut().unwrap().render_area;
        render_area.width = width;
        render_area.height = height;
        Ok(())
    }

    pub fn renderpass_init(&mut self) -> Result<(), EngineError> {
        // TODO: make the renderpass initialization configurable
        let render_area = RenderArea {
            x: 0.,
            y: 0.,
            width: self.framebuffer_width as f32,
            height: self.framebuffer_height as f32,
        };
        let clear_color = Color::default();
        let depth = 1.;
        let stencil = 0;

        // Main subpass
        let subpass =
            SubpassDescription::default().pipeline_bind_point(PipelineBindPoint::GRAPHICS);

        // Attachments
        // TODO: make the renderpass attachments configurable
        // Color attachment
        let color_attachment = self.init_color_attachment()?;
        let color_attachment_reference = [AttachmentReference::default()
            .attachment(0) // Attachment description array index
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpass = subpass.color_attachments(&color_attachment_reference);
        // Depth attachment, if there is one
        let depth_attachment = self.init_depth_attachment()?;
        let has_depth = depth_attachment.is_some();
        let depth_attachment_reference = AttachmentReference::default()
            .attachment(1) // Attachment description array index
            .layout(ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = if let Some(depth_attachment) = depth_attachment {
            subpass.depth_stencil_attachment(&depth_attachment_reference)
        } else {
            subpass
        };
        // TODO: other attachment types (input, resolve, preserve)

        // Dependencies
        let dependencies = [self.init_dependencies()?];
        let subpass = [subpass];
        // Render pass create
        let renderpass_info = RenderPassCreateInfo::default()
            .subpasses(&subpass)
            .dependencies(&dependencies);

        let attachments = [color_attachment];
        let attachments_with_depth = if has_depth {
            Some([color_attachment, depth_attachment.unwrap()])
        } else {
            None
        };
        let renderpass_info = if has_depth {
            renderpass_info.attachments(attachments_with_depth.as_ref().unwrap())
        } else {
            renderpass_info.attachments(&attachments)
        };

        let device = self.get_device()?;
        let renderpass = unsafe {
            match device.create_render_pass(&renderpass_info, self.get_allocator()?) {
                Ok(renderpass) => renderpass,
                Err(err) => {
                    error!("Failed to create the vuklan renderpass: {:?}", err);
                    return Err(EngineError::InitializationFailed);
                }
            }
        };

        self.context.renderpass = Some(Renderpass {
            handler: renderpass,
            render_area,
            clear_color,
            depth,
            stencil,
            state: RenderpassState::Ready,
        });

        Ok(())
    }

    pub fn renderpass_shutdown(&mut self) -> Result<(), EngineError> {
        let device = self.get_device()?;
        unsafe {
            device.destroy_render_pass(self.get_renderpass()?.handler, self.get_allocator()?);
        };
        Ok(())
    }

    /// None if there swapchain needs to be recreated
    pub fn renderpass_begin(
        &self,
        command_buffer: &CommandBuffer,
        frame_buffer: Framebuffer,
    ) -> Result<(), EngineError> {
        let renderpass = self.get_renderpass()?;
        let render_area_offset = Offset2D {
            x: renderpass.render_area.x as i32,
            y: renderpass.render_area.y as i32,
        };

        let render_area_extent = Extent2D {
            width: renderpass.render_area.width as u32,
            height: renderpass.render_area.height as u32,
        };

        if render_area_extent.width > self.framebuffer_width
            || render_area_extent.height > self.framebuffer_height
        {
            error!("Could not begin the renderpass, the render area ({:?}, {:?}) is bigger than the framebuffer ({:?}, {:?})",
        render_area_extent.width, render_area_extent.height, self.framebuffer_width, self.framebuffer_height);
            return Err(EngineError::InvalidValue);
        }

        let clear_values_color: ClearValue = ClearValue {
            color: ClearColorValue {
                float32: [
                    renderpass.clear_color.r,
                    renderpass.clear_color.g,
                    renderpass.clear_color.b,
                    renderpass.clear_color.a,
                ],
            },
        };
        let clear_values_depth: ClearValue = ClearValue {
            depth_stencil: ClearDepthStencilValue {
                depth: renderpass.depth,
                stencil: renderpass.stencil,
            },
        };
        let clear_values = [clear_values_color, clear_values_depth];

        let renderpass_begin_info = RenderPassBeginInfo::default()
            .render_pass(renderpass.handler)
            .framebuffer(frame_buffer)
            .render_area(Rect2D {
                offset: render_area_offset,
                extent: render_area_extent,
            })
            .clear_values(&clear_values);

        let device = self.get_device()?;
        unsafe {
            device.cmd_begin_render_pass(
                *command_buffer.handler.as_ref(),
                &renderpass_begin_info,
                SubpassContents::INLINE,
            )
        };

        Ok(())
    }

    pub fn renderpass_end(&self, command_buffer: &CommandBuffer) -> Result<(), EngineError> {
        let device = self.get_device()?;
        unsafe {
            device.cmd_end_render_pass(*command_buffer.handler.as_ref());
        };
        Ok(())
    }

    pub fn get_renderpass(&self) -> Result<&Renderpass, EngineError> {
        match &self.context.renderpass {
            Some(renderpass) => Ok(renderpass),
            None => {
                error!("Can't access the vulkan renderpass");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
