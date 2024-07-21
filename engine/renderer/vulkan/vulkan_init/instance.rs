use std::ffi::{CStr, CString};

use ash::vk::{make_api_version, ApplicationInfo, InstanceCreateInfo, API_VERSION_1_3};

use crate::{
    core::debug::{errors::EngineError}, error, platforms::platform::Platform,
    renderer::vulkan::vulkan_types::VulkanRendererBackend,
    debug
};

impl VulkanRendererBackend<'_> {
    pub fn get_instance(&self) -> Result<&ash::Instance, EngineError> {
        match &self.context.instance {
            Some(instance) => Ok(instance),
            None => {
                error!("Can't access the vulkan instance");
                Err(EngineError::AccessFailed)
            }
        }
    }

    fn get_required_layers(&self) -> Result<Vec<*const i8>, EngineError> {
        let mut required_layers = Vec::new();

        #[cfg(debug_assertions)]
        required_layers.push(
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }
                .as_ptr(),
        );

        let available_layers = unsafe {
            match self.get_entry()?.enumerate_instance_layer_properties() {
                Ok(layers) => layers,
                Err(err) => {
                    error!("Failed to enumerate the available layers: {:?}", err);
                    return Err(EngineError::InitializationFailed);
                }
            }
        };
        for required in required_layers.clone() {
            let mut is_available = false;
            'inner: for available in &available_layers {
                let name = match available.layer_name_as_c_str() {
                    Ok(name) => name,
                    Err(err) => {
                        error!("Failed to fetch the layer name: {:?}", err);
                        return Err(EngineError::InitializationFailed);
                    }
                };
                if name == unsafe { CStr::from_ptr(required) } {
                    is_available = true;
                    break 'inner;
                }
            }
            if !is_available {
                error!("The required layer {:?} is not available!\n", required);
                return Err(EngineError::VulkanFailed);
            }
        }
        Ok(required_layers)
    }

    fn get_required_extensions(
        &self,
        platform: &dyn Platform,
    ) -> Result<Vec<*const i8>, EngineError> {
        let mut required_extensions = platform.get_required_extensions()?;
        required_extensions.push(unsafe {
            CStr::from_bytes_with_nul_unchecked(b"VK_KHR_surface\0").as_ptr()
        });
        
        #[cfg(debug_assertions)]
        required_extensions.push(unsafe {
            CStr::from_bytes_with_nul_unchecked(b"VK_EXT_debug_utils\0").as_ptr()
        });

        Ok(required_extensions)
    }

    fn display_extensions(extensions: &Vec<*const i8>){
        debug!("Extensions:");
        for extension in extensions {
            let extension_name = unsafe { CStr::from_ptr(*extension).to_string_lossy() };
            debug!("\t{:?}", extension_name);
        }
    }

    fn display_layers(layers: &Vec<*const i8>){
        debug!("Layers:");
        for layer in layers {
            let layer_name = unsafe { CStr::from_ptr(*layer).to_string_lossy() };
            debug!("\t{:?}", layer_name);
        }
    }

    pub fn instance_init(
        &mut self,
        application_name: &str,
        platform: &dyn Platform,
    ) -> Result<(), EngineError> {
        let engine_name_cstr = CString::new("BigoudiEngine").unwrap();
        let application_name_cstr = CString::new(application_name).unwrap();

        let application_info = ApplicationInfo::default()
            .api_version(API_VERSION_1_3)
            .application_name(&application_name_cstr)
            .application_version(make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name_cstr)
            .engine_version(make_api_version(0, 1, 0, 0));

        // Get the required extensions
        let required_extensions = self.get_required_extensions(platform)?;

        // Get the required layers
        let required_layers = self.get_required_layers()?;

        #[cfg(debug_assertions)]
        Self::display_extensions(&required_extensions);
        
        #[cfg(debug_assertions)]
        Self::display_layers(&required_layers);

        let instance_create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&required_extensions)
            .enabled_layer_names(&required_layers);

        unsafe {
            match self
                .get_entry()?
                .create_instance(&instance_create_info, self.get_allocator()?)
            {
                Ok(instance) => {
                    self.context.instance = Some(instance);
                    Ok(())
                }
                Err(err) => {
                    error!("Failed to create the vulkan instance: {:?}", err);
                    Err(EngineError::VulkanFailed)
                }
            }
        }
    }

    pub fn instance_shutdown(&mut self) -> Result<(), EngineError> {
        unsafe {
            self.get_instance()?.destroy_instance(self.get_allocator()?);
        }
        Ok(())
    }
}
