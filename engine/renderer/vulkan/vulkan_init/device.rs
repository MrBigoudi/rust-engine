use std::ffi::CStr;

use ash::{vk::{api_version_major, api_version_minor, api_version_patch, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, PhysicalDeviceType, QueueFlags}, Device};

use crate::{
    core::debug::errors::EngineError, debug, error, renderer::vulkan::vulkan_types::VulkanRendererBackend
};

use super::swapchain::SwapChainSupportDetails;

struct PhysicalDeviceRequirements {
    does_require_graphics_queue: bool,
    does_require_present_queue: bool,
    does_require_compute_queue: bool,
    does_require_transfer_queue: bool,
    does_require_sampler_anisotropy: bool,
    is_discrete_gpu: bool,
    device_extensions: Vec<*const i8>,
}

impl Default for PhysicalDeviceRequirements {
    fn default() -> Self {
        let mut device_extensions = Vec::new();
        device_extensions.push(
            unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_KHR_swapchain\0").as_ptr() }
        );
        
        Self { 
            does_require_graphics_queue: true, 
            does_require_present_queue: true, 
            does_require_compute_queue: true, 
            does_require_transfer_queue: true, 
            does_require_sampler_anisotropy: true, 
            is_discrete_gpu: false, 
            device_extensions, 
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct PhysicalDeviceInfo {
    pub graphics_family_index: Option<usize>,
    pub present_family_index: Option<usize>,
    pub compute_family_index: Option<usize>,
    pub transfer_family_index: Option<usize>,
    pub properties: Option<PhysicalDeviceProperties>,
    pub features: Option<PhysicalDeviceFeatures>,
    pub memory_properties: Option<PhysicalDeviceMemoryProperties>,
}

impl VulkanRendererBackend<'_> {
    fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, EngineError> {
        match unsafe { self.get_instance()?.enumerate_physical_devices() } {
            Ok(physical_devices) => Ok(physical_devices),
            Err(err) => {
                error!("Failed to enumerate the available physical devices: {:?}", err);
                Err(EngineError::VulkanFailed)
            }
        }
    }

    fn query_swapchain_support(&self, physical_device: &PhysicalDevice) -> Result<SwapChainSupportDetails, EngineError> {
        let surface_capabilities = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_capabilities(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        let surface_format = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_formats(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        let surface_present_modes = unsafe {
            self.get_surface_loader()?
                .get_physical_device_surface_present_modes(*physical_device, *(self.get_surface()?))
                .unwrap()
        };

        Ok(SwapChainSupportDetails {
            capabilities: surface_capabilities,
            formats: surface_format,
            present_modes: surface_present_modes,
        })
    }

    fn are_queue_families_requirements_fullfiled(requirements: &PhysicalDeviceRequirements, device_info: &PhysicalDeviceInfo) -> bool {
        (!requirements.does_require_graphics_queue || (requirements.does_require_graphics_queue && device_info.graphics_family_index.is_some()))
        && (!requirements.does_require_present_queue || (requirements.does_require_present_queue && device_info.present_family_index.is_some()))
        && (!requirements.does_require_compute_queue || (requirements.does_require_compute_queue && device_info.graphics_family_index.is_some()))
        && (!requirements.does_require_transfer_queue || (requirements.does_require_transfer_queue && device_info.graphics_family_index.is_some()))
    }

    fn are_swapchain_requirements_fullfiled(&self, physical_device: &PhysicalDevice) -> Result<bool, EngineError> {
        let swapchain_supprt_details = match self.query_swapchain_support(physical_device){
            Ok(details) => details,
            Err(err) => {
                error!("Failed to query the swapchain support details: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        };
        
        Ok(swapchain_supprt_details.is_complete())
    }

    fn are_extensions_requirements_fullfiled(&self, physical_device: &PhysicalDevice, requirements: &PhysicalDeviceRequirements) -> Result<bool, EngineError> {
        let extension_properties = match unsafe {self.get_instance()?.enumerate_device_extension_properties(*physical_device)} {
            Ok(properties) => properties,
            Err(err) => {
                error!("Failed to enumerate the device extension properties: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        };

        'cur_extension: for required_extension in &requirements.device_extensions {
            let required_extension_cstr = unsafe { CStr::from_ptr(required_extension.clone()) };
            for found_extension in &extension_properties {
                let found_extension_cstr =unsafe { CStr::from_ptr(found_extension.extension_name.as_ptr()) };
                if found_extension_cstr == required_extension_cstr {
                    continue 'cur_extension;
                }
            }
            return Ok(false);
        }

        Ok(true)
    }

    fn is_device_suitable(&self, physical_device: &PhysicalDevice, requirements: &PhysicalDeviceRequirements) -> Result<(bool, Option<PhysicalDeviceInfo>), EngineError> {
        let properties = unsafe { self.get_instance()?.get_physical_device_properties(*physical_device) };
        let features = unsafe { self.get_instance()?.get_physical_device_features(*physical_device) };
        let memory_properties = unsafe { self.get_instance()?.get_physical_device_memory_properties(*physical_device) };

        // Discrete GPU ?
        if requirements.is_discrete_gpu {
            if properties.device_type != PhysicalDeviceType::DISCRETE_GPU {
                debug!("Device should be a discrete GPU, found `{:?}' instead", properties.device_type);
                return Ok((false, None));
            }
        }

        // Anisotropy ?
        if requirements.does_require_sampler_anisotropy {
            if features.sampler_anisotropy == 0 {
                debug!("Device should support sampler anisotropy");
                return Ok((false, None));
            }
        }

        let mut queue_families_info = PhysicalDeviceInfo::default();
        queue_families_info.properties = Some(properties);
        queue_families_info.features = Some(features);
        queue_families_info.memory_properties = Some(memory_properties);

        let queue_family_properties = unsafe { self.get_instance()?.get_physical_device_queue_family_properties(*physical_device)};
        
        let mut min_transfer_score = u32::max_value();
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            let mut transfer_score = 0;
            
            // Graphics queue ?
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                queue_families_info.graphics_family_index = Some(index);
                transfer_score += 1;
            }

            // Compute queue ?
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE) {
                queue_families_info.compute_family_index = Some(index);
                transfer_score += 1;
            }

            // Transfer queue ?
            if queue_family.queue_flags.contains(QueueFlags::TRANSFER) {
                // Take the index if it is the current lowest. This increases the
                // likelihood that it is a dedicated transfer queue.
                if transfer_score <= min_transfer_score {
                    min_transfer_score = transfer_score;
                    queue_families_info.transfer_family_index = Some(index);
                }
            }

            // Present queue ?
            match unsafe { self.get_surface_loader()?.get_physical_device_surface_support(
                *physical_device,
                index as u32,
                *self.get_surface()?,
            ) } {
                Ok(false) => (),
                Ok(true) => queue_families_info.present_family_index = Some(index),
                Err(err) => {
                    error!("Failed to fetch the physical device surface support: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
            }

        }

        let are_queue_families_requirements_fullfiled = Self::are_queue_families_requirements_fullfiled(requirements, &queue_families_info);
        let are_swapchain_requirements_fullfiled = self.are_swapchain_requirements_fullfiled(physical_device)?;
        let are_extensions_requirements_fullfiled = self.are_extensions_requirements_fullfiled(physical_device, requirements)?;

        let is_device_suitable = are_queue_families_requirements_fullfiled
            && are_swapchain_requirements_fullfiled
            && are_extensions_requirements_fullfiled;

        Ok((is_device_suitable, Some(queue_families_info)))
    }

    fn display_physical_device(physical_device: &PhysicalDevice, device_info: &PhysicalDeviceInfo) {
        if let Some(properties) = &device_info.properties {
            // Convert the device name array to a raw pointer
            let name_ptr = properties.device_name.as_ptr();
            let name = unsafe { CStr::from_ptr(name_ptr) };
            debug!("\tSelected device: {:?}", name);

            // GPU type, etc.
            match properties.device_type {
                PhysicalDeviceType::CPU => debug!("\tGPU type is CPU"),
                PhysicalDeviceType::DISCRETE_GPU => debug!("\tGPU type is discrete"),
                PhysicalDeviceType::INTEGRATED_GPU => debug!("\tGPU type is integrated"),
                PhysicalDeviceType::OTHER => debug!("\tGPU type is unknown"),
                PhysicalDeviceType::VIRTUAL_GPU => debug!("\tGPU type is virtual"),
                _ => (),
            }

            debug!("\tGPU Driver version: {:?}.{:?}.{:?}",
                api_version_major(properties.driver_version),
                api_version_minor(properties.driver_version),
                api_version_patch(properties.driver_version),
            );

            debug!("\tVulkan API version: {:?}.{:?}.{:?}\n\n",
                api_version_major(properties.api_version),
                api_version_minor(properties.api_version),
                api_version_patch(properties.api_version),
            );
        }
    }

    pub fn physical_device_init(&mut self) -> Result<(), EngineError> {
        let physical_devices = self.enumerate_physical_devices()?;

        // TODO: make this configurable
        let requirements = PhysicalDeviceRequirements::default();

        for physical_device in physical_devices {
            let (is_suitable, device_info) = match self.is_device_suitable(&physical_device, &requirements) {
                Ok((true, Some(info))) => (true, info),
                Ok((false, _)) => (false, PhysicalDeviceInfo::default()),
                Err(err) => {
                    error!("Faile to get the suitability of the current physical device: {:?}", err);
                    return Err(EngineError::VulkanFailed);
                }
                _ => {
                    error!("Faile to get the suitability of the current physical device");
                    return Err(EngineError::Unknown);
                }
            };

            if is_suitable {
                debug!("Found physical device");
                Self::display_physical_device(&physical_device, &device_info);
                self.context.physical_device = Some(physical_device);
                self.context.physical_device_info = Some(device_info);
                return Ok(())
            }
        }

        error!("Failed to find a suitable physical device");
        Err(EngineError::VulkanFailed)
    }

    pub fn physical_device_shutdown(&mut self) -> Result<(), EngineError> {
        Ok(())
    }

    pub fn get_physical_device(&self) -> Result<&PhysicalDevice, EngineError> {
        match &self.context.physical_device {
            Some(device) => Ok(device),
            None => {
                error!("Can't access the vulkan physical device");
                Err(EngineError::AccessFailed)
            }
        }
    }

    pub fn device_init(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn device_shutdown(&mut self) -> Result<(), EngineError> {
        todo!()
    }

    pub fn get_device(&self) -> Result<&Device, EngineError> {
        match &self.context.device {
            Some(device) => Ok(device),
            None => {
                error!("Can't access the vulkan device");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
