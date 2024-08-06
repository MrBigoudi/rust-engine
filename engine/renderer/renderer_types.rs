pub(crate) enum RendererBackendType {
    Vulkan,
    OpenGl,
    DirectX,
}

pub(crate) struct RenderFrameData {
    pub delta_time: f64,
}
