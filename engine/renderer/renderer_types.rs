pub(crate) enum RendererBackendType {
    Vulkan,
    OpenGl,
    DirectX,
}

pub(crate) struct RenderFrameData {
    pub delta_time: f64,
}

pub(crate) struct RendererGlobalUniformObject {
    pub projection: glam::Mat4,  // 64 bytes
    pub view: glam::Mat4,        // 64 bytes
    pub reserved_01: glam::Mat4, // 64 bytes
    pub reserved_02: glam::Mat4, // 64 bytes
}

impl Default for RendererGlobalUniformObject {
    fn default() -> Self {
        Self {
            projection: glam::Mat4::IDENTITY,
            view: glam::Mat4::IDENTITY,
            reserved_01: glam::Mat4::ZERO,
            reserved_02: glam::Mat4::ZERO,
        }
    }
}
