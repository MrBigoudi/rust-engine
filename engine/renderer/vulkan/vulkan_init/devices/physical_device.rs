use std::ffi::CStr;

use ash::vk::{
    api_version_major, api_version_minor, api_version_patch, ExtensionProperties, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, PhysicalDeviceType, QueueFlags
};

use crate::{
    core::debug::errors::EngineError, debug, error,
    renderer::vulkan::{vulkan_init::swapchain::SwapChainSupportDetails, vulkan_types::VulkanRendererBackend, vulkan_utils::physical_device_features_to_vector},
};

use super::device_requirements::DeviceRequirements;



#[derive(Default, Debug)]
pub(crate) struct PhysicalDeviceInfo {
    pub graphics_family_index: Option<usize>,
    pub graphics_family_queue_count: Option<u32>,
    pub present_family_index: Option<usize>,
    pub present_family_queue_count: Option<u32>,
    pub compute_family_index: Option<usize>,
    pub compute_family_queue_count: Option<u32>,
    pub transfer_family_index: Option<usize>,
    pub transfer_family_queue_count: Option<u32>,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub extension_properties: Vec<ExtensionProperties>,
    pub memory_properties: PhysicalDeviceMemoryProperties,
}

impl VulkanRendererBackend<'_> {
    fn enumerate_physical_devices(&self) -> Result<Vec<PhysicalDevice>, EngineError> {
        match unsafe { self.get_instance()?.enumerate_physical_devices() } {
            Ok(physical_devices) => Ok(physical_devices),
            Err(err) => {
                error!(
                    "Failed to enumerate the available physical devices: {:?}",
                    err
                );
                Err(EngineError::VulkanFailed)
            }
        }
    }

    fn query_swapchain_support(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<SwapChainSupportDetails, EngineError> {
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

    fn are_queue_families_requirements_fullfiled(
        requirements: &DeviceRequirements,
        device_info: &PhysicalDeviceInfo,
    ) -> bool {
        !(requirements.does_require_graphics_queue && device_info.graphics_family_index.is_none()
            || requirements.does_require_present_queue
                && device_info.present_family_index.is_none()
            || requirements.does_require_compute_queue
                && device_info.graphics_family_index.is_none()
            || requirements.does_require_transfer_queue
                && device_info.graphics_family_index.is_none())
    }

    fn are_swapchain_requirements_fullfiled(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<bool, EngineError> {
        let swapchain_supprt_details = match self.query_swapchain_support(physical_device) {
            Ok(details) => details,
            Err(err) => {
                error!("Failed to query the swapchain support details: {:?}", err);
                return Err(EngineError::VulkanFailed);
            }
        };

        Ok(swapchain_supprt_details.is_complete())
    }

    fn are_extensions_requirements_fullfiled(
        &self,
        requirements: &DeviceRequirements,
        physical_device_info: &PhysicalDeviceInfo
    ) -> Result<bool, EngineError> {
        'cur_extension: for required_extension in &requirements.extensions {
            let required_extension_cstr = unsafe { CStr::from_ptr(*required_extension) };
            for found_extension in &physical_device_info.extension_properties {
                let found_extension_cstr =
                    unsafe { CStr::from_ptr(found_extension.extension_name.as_ptr()) };
                if found_extension_cstr == required_extension_cstr {
                    continue 'cur_extension;
                }
            }
            return Ok(false);
        }
        Ok(true)
    }

    fn are_features_requirements_fullfiled(
        &self,
        requirements: &DeviceRequirements,
        physical_device_info: &PhysicalDeviceInfo
    ) -> Result<bool, EngineError> {
        let physical_device_features = &physical_device_info.features;
        let required_features_as_vec = physical_device_features_to_vector(&requirements.features);
        let features_as_vec = physical_device_features_to_vector(physical_device_features);
        if required_features_as_vec.len() != features_as_vec.len() {
            error!("The required features and the physical device features are incompatible !");
            return Err(EngineError::Unknown);
        }
        let nb_features = features_as_vec.len();
        for feature in 0..nb_features {
            if required_features_as_vec[feature].1 && !features_as_vec[feature].1 {
                debug!("Device should support {:?}", required_features_as_vec[feature].0);
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn physical_device_info_init(&self, physical_device: &PhysicalDevice) -> Result<PhysicalDeviceInfo, EngineError> {
        let properties = unsafe {
            self.get_instance()?
                .get_physical_device_properties(*physical_device)
        };
        let features = unsafe {
            self.get_instance()?
                .get_physical_device_features(*physical_device)
        };
        let memory_properties = unsafe {
            self.get_instance()?
                .get_physical_device_memory_properties(*physical_device)
        };
        let extension_properties = match unsafe {
            self.get_instance()?
                .enumerate_device_extension_properties(*physical_device)
        } {
            Ok(properties) => properties,
            Err(err) => {
                error!(
                    "Failed to enumerate the physical device extension properties: {:?}",
                    err
                );
                return Err(EngineError::VulkanFailed);
            }
        };

        Ok(PhysicalDeviceInfo {
            properties,
            features,
            extension_properties,
            memory_properties,
            graphics_family_index: None,
            graphics_family_queue_count: None,
            present_family_index: None,
            present_family_queue_count: None,
            compute_family_index: None,
            compute_family_queue_count: None,
            transfer_family_index: None,
            transfer_family_queue_count: None,
        })
    }

    /// Modify the physical device info
    fn queue_family_properties_init(
        &self, 
        physical_device: &PhysicalDevice, 
        physical_device_info: &mut PhysicalDeviceInfo
    ) -> Result<(), EngineError> {
        let queue_family_properties = unsafe {
            self.get_instance()?
                .get_physical_device_queue_family_properties(*physical_device)
        };

        let mut min_transfer_score = u32::MAX;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            let mut transfer_score = 0;

            // Graphics queue ?
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                physical_device_info.graphics_family_index = Some(index);
                physical_device_info.graphics_family_queue_count = Some(queue_family.queue_count);
                transfer_score += 1;
            }

            // Compute queue ?
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE) {
                physical_device_info.compute_family_index = Some(index);
                physical_device_info.compute_family_queue_count = Some(queue_family.queue_count);
                transfer_score += 1;
            }

            // Transfer queue ?
            if queue_family.queue_flags.contains(QueueFlags::TRANSFER) {
                // Take the index if it is the current lowest. This increases the
                // likelihood that it is a dedicated transfer queue.
                if transfer_score <= min_transfer_score {
                    min_transfer_score = transfer_score;
                    physical_device_info.transfer_family_index = Some(index);
                    physical_device_info.transfer_family_queue_count = Some(queue_family.queue_count);
                }
            }

            // Present queue ?
            match unsafe {
                self.get_surface_loader()?
                    .get_physical_device_surface_support(
                        *physical_device,
                        index as u32,
                        *self.get_surface()?,
                    )
            } {
                Ok(false) => (),
                Ok(true) => {
                    physical_device_info.present_family_index = Some(index);
                    physical_device_info.present_family_queue_count = Some(queue_family.queue_count);
                },
                Err(err) => {
                    error!(
                        "Failed to fetch the physical device surface support: {:?}",
                        err
                    );
                    return Err(EngineError::VulkanFailed);
                }
            }
        }
        Ok(())
    }

    fn is_device_suitable(
        &self,
        physical_device: &PhysicalDevice,
        requirements: &DeviceRequirements,
    ) -> Result<(bool, Option<PhysicalDeviceInfo>), EngineError> {
        let mut physical_device_info = self.physical_device_info_init(physical_device)?;
        self.queue_family_properties_init(physical_device, &mut physical_device_info)?;

        // Discrete GPU ?
        if requirements.is_discrete_gpu
            && physical_device_info.properties.device_type != PhysicalDeviceType::DISCRETE_GPU
        {
            debug!(
                "Device should be a discrete GPU, found `{:?}' instead",
                physical_device_info.properties.device_type
            );
            return Ok((false, None));
        }

        let are_queue_families_requirements_fullfiled =
            Self::are_queue_families_requirements_fullfiled(requirements, &physical_device_info);
        let are_swapchain_requirements_fullfiled =
            self.are_swapchain_requirements_fullfiled(physical_device)?;
        let are_extensions_requirements_fullfiled =
            self.are_extensions_requirements_fullfiled(requirements, &physical_device_info)?;
        let are_features_requirements_fullfiled =
            self.are_features_requirements_fullfiled(requirements, &physical_device_info)?;

        let is_device_suitable = are_queue_families_requirements_fullfiled
            && are_swapchain_requirements_fullfiled
            && are_extensions_requirements_fullfiled
            && are_features_requirements_fullfiled;

        Ok((is_device_suitable, Some(physical_device_info)))
    }

    fn display_physical_device(physical_device: &PhysicalDevice, device_info: &PhysicalDeviceInfo) {
        // Convert the device name array to a raw pointer
        let name_ptr = device_info.properties.device_name.as_ptr();
        let name = unsafe { CStr::from_ptr(name_ptr) };
        debug!("\tSelected device: {:?}", name);

        // GPU type, etc.
        match device_info.properties.device_type {
            PhysicalDeviceType::CPU => debug!("\tGPU type is CPU"),
            PhysicalDeviceType::DISCRETE_GPU => debug!("\tGPU type is discrete"),
            PhysicalDeviceType::INTEGRATED_GPU => debug!("\tGPU type is integrated"),
            PhysicalDeviceType::OTHER => debug!("\tGPU type is unknown"),
            PhysicalDeviceType::VIRTUAL_GPU => debug!("\tGPU type is virtual"),
            _ => (),
        }

        debug!(
            "\tGPU Driver version: {:?}.{:?}.{:?}",
            api_version_major(device_info.properties.driver_version),
            api_version_minor(device_info.properties.driver_version),
            api_version_patch(device_info.properties.driver_version),
        );

        debug!(
            "\tVulkan API version: {:?}.{:?}.{:?}\n\n",
            api_version_major(device_info.properties.api_version),
            api_version_minor(device_info.properties.api_version),
            api_version_patch(device_info.properties.api_version),
        );
    }

    pub fn physical_device_init(&mut self) -> Result<(), EngineError> {
        let physical_devices = self.enumerate_physical_devices()?;

        let requirements = self.get_device_requirements()?;

        for physical_device in physical_devices {
            let (is_suitable, device_info) =
                match self.is_device_suitable(&physical_device, requirements) {
                    Ok((true, Some(info))) => (true, info),
                    Ok((false, _)) => (false, PhysicalDeviceInfo::default()),
                    Err(err) => {
                        error!(
                            "Failed to get the suitability of the current physical device: {:?}",
                            err
                        );
                        return Err(EngineError::VulkanFailed);
                    }
                    _ => {
                        error!("Failed to get the suitability of the current physical device");
                        return Err(EngineError::Unknown);
                    }
                };

            if is_suitable {
                debug!("Found physical device");
                Self::display_physical_device(&physical_device, &device_info);
                self.context.physical_device = Some(physical_device);
                self.context.physical_device_info = Some(device_info);
                return Ok(());
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

    pub fn get_physical_device_info(&self) -> Result<&PhysicalDeviceInfo, EngineError> {
        match &self.context.physical_device_info {
            Some(device_info) => Ok(device_info),
            None => {
                error!("Can't access the vulkan physical device info");
                Err(EngineError::AccessFailed)
            }
        }
    }
}
