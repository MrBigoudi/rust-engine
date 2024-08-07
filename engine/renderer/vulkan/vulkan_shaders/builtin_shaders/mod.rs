pub mod object_shaders;

use ash::{vk, Device};
use object_shaders::ObjectShaders;

use crate::core::debug::errors::EngineError;

pub(crate) struct BuiltinShaders {
    pub object_shaders: ObjectShaders,
}

impl BuiltinShaders {
    pub fn create(
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<Self, EngineError> {
        let object_shaders = ObjectShaders::create(device, allocator)?;
        Ok(BuiltinShaders { object_shaders })
    }

    pub fn destroy(
        &self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        self.object_shaders.destroy(device, allocator)?;
        Ok(())
    }
}
