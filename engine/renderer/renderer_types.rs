pub(crate) enum RendererBackendType {
    Vulkan,
    OpenGl,
    DirectX,
}

pub(crate) struct RenderFrame {
    pub delta_time: f64,
}
