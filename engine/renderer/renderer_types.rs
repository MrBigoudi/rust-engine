use crate::{resources::texture::Texture, warn};

pub(crate) enum RendererBackendType {
    Vulkan,
    OpenGl,
    DirectX,
}

pub(crate) struct RenderFrameData {
    pub delta_time: f64,
}

/// Max 3 for triple-buffering
pub const RENDERER_MAX_IN_FLIGHT_FRAMES: usize = 3;

/// Uploaded once per frame
#[repr(C)]
pub(crate) struct RendererGlobalUniformObject {
    pub projection: glam::Mat4,  // 64 bytes
    pub view: glam::Mat4,        // 64 bytes
    pub reserved_01: glam::Mat4, // 64 bytes reserved for future use
    pub reserved_02: glam::Mat4, // 64 bytes reserved for future use
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

/// Uploaded once per object per frame
#[repr(C)]
pub(crate) struct RendererPerObjectUniformObject {
    pub diffuse: glam::Vec4,     // 16 bytes
    pub reserved_01: glam::Vec4, // 16 bytes reserved for future use
    pub reserved_02: glam::Vec4, // 16 bytes reserved for future use
    pub reserved_03: glam::Vec4, // 16 bytes reserved for future use
}

impl RendererPerObjectUniformObject {
    pub fn diffuse(mut self, diffuse: glam::Vec4) -> Self {
        self.diffuse = diffuse;
        self
    }
}

impl Default for RendererPerObjectUniformObject {
    fn default() -> Self {
        Self {
            diffuse: glam::Vec4::ONE,
            reserved_01: glam::Vec4::ZERO,
            reserved_02: glam::Vec4::ZERO,
            reserved_03: glam::Vec4::ZERO,
        }
    }
}

pub const RENDERER_MAX_NUMBER_OF_TEXTURES_PER_OBJECT: usize = 16;

pub(crate) struct GeometryRenderData {
    pub object_id: Option<u32>,
    pub model: glam::Mat4,
    pub textures: [Option<Box<dyn Texture>>; RENDERER_MAX_NUMBER_OF_TEXTURES_PER_OBJECT],
}

impl GeometryRenderData {
    pub fn model(mut self, model: glam::Mat4) -> Self {
        self.model = model;
        self
    }
    pub fn object_id(mut self, id: Option<u32>) -> Self {
        self.object_id = id;
        self
    }
    pub fn textures(
        mut self,
        textures: [Option<Box<dyn Texture>>; RENDERER_MAX_NUMBER_OF_TEXTURES_PER_OBJECT],
    ) -> Self {
        self.textures = textures;
        self
    }
    pub fn texture(mut self, index: usize, texture: Option<Box<dyn Texture>>) -> Self {
        if index >= RENDERER_MAX_NUMBER_OF_TEXTURES_PER_OBJECT {
            warn!("The index of the texture set for the geometry render data is too big, setup cancelled");
            return self;
        }
        self.textures[index] = texture;
        self
    }
}

impl Default for GeometryRenderData {
    fn default() -> Self {
        Self {
            object_id: None,
            model: glam::Mat4::IDENTITY,
            textures: Default::default(),
        }
    }
}

#[repr(C)]
pub(crate) struct VertexData {
    pub position: glam::Vec3,
    pub texture: glam::Vec2,
}
