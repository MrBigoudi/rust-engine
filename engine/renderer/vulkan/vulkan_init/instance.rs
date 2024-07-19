use std::ffi::CString;

use ash::vk::{make_api_version, ApplicationInfo, InstanceCreateInfo, API_VERSION_1_3};

use crate::{
    core::errors::EngineError, error, platforms::platform::Platform,
    renderer::vulkan::vulkan_types::VulkanRendererBackend,
};

impl VulkanRendererBackend<'_> {
    pub fn init_instance(
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

        let instance_create_info =
            InstanceCreateInfo::default().application_info(&application_info);

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
}
