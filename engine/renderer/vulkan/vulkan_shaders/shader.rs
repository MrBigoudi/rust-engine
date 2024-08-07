use std::path::Path;

use ash::{
    util::read_spv,
    vk::{self, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags},
    Device,
};

use crate::{core::debug::errors::EngineError, error};

pub(crate) struct Shader {
    pub shader_module: ShaderModule,
    pub stage_flag: ShaderStageFlags,
    pub entry_point: String,
}

impl Shader {
    fn get_compiled_shader_path(shader: &str) -> String {
        let base_path = Path::new("/target/assets/shaders");
        let relative_path = Path::new(shader);
        base_path
            .join(relative_path)
            .with_extension("spv")
            .to_string_lossy()
            .into_owned()
    }

    /// Create a shader stage
    /// device The logical device to build the shader module
    /// stage_flag Indicates the type of shader (Vertex, Fragment, ...)
    /// shader_path_from_shaders_dir The shader path within the assets/shaders/ folder (expect .slang file)
    /// shader_entry_point The name of the entry point function for the shader stage, if None default to "main"
    pub fn create(
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
        stage_flag: ShaderStageFlags,
        shader_path_from_shaders_dir: &str,
        shader_entry_point: Option<&str>,
    ) -> Result<Self, EngineError> {
        let crate_path = env!("CARGO_MANIFEST_DIR");
        let spv_path =
            crate_path.to_owned() + &Self::get_compiled_shader_path(shader_path_from_shaders_dir);
        // open the file. With cursor at the end
        let mut spv_file = match std::fs::File::open(spv_path.clone()) {
            Ok(file) => file,
            Err(err) => {
                error!(
                    "Failed to open the vulkan shader {:?}: {:?}",
                    spv_path, err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        let spv_code = match read_spv(&mut spv_file) {
            Ok(code) => code,
            Err(err) => {
                error!(
                    "Failed to read the vulkan shader {:?}: {:?}",
                    spv_path, err
                );
                return Err(EngineError::InitializationFailed);
            }
        };

        let create_info = ShaderModuleCreateInfo::default().code(&spv_code);

        let shader_module = unsafe {
            match device.create_shader_module(&create_info, allocator) {
                Ok(module) => module,
                Err(err) => {
                    error!(
                        "Failed to create a vulkan shader module for shader {:?}: {:?}",
                        spv_path, err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        };

        let entry_point = match shader_entry_point {
            Some(entry) => String::from(entry),
            None => String::from("main"),
        };

        Ok(Shader {
            shader_module,
            stage_flag,
            entry_point,
        })
    }

    pub fn destroy(
        &self,
        device: &Device,
        allocator: Option<&vk::AllocationCallbacks<'_>>,
    ) -> Result<(), EngineError> {
        unsafe {
            device.destroy_shader_module(self.shader_module, allocator);
        }
        Ok(())
    }
}
