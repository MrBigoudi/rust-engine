pub mod object_shaders;

use ash::Device;
use object_shaders::ObjectShaders;

use crate::core::debug::errors::EngineError;


pub(crate) struct BuiltinShaders {
    pub object_shaders: ObjectShaders,
}

impl BuiltinShaders {
    pub fn create(device: &Device) -> Result<Self, EngineError> {
        let object_shaders = ObjectShaders::create(device)?;
        Ok(BuiltinShaders{
            object_shaders,
        })
    }
}