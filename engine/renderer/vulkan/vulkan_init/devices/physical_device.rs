use std::ffi::CStr;

use ash::vk::{
    api_version_major, api_version_minor, api_version_patch, ExtensionProperties, Format,
    FormatFeatureFlags, MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceMemoryProperties, PhysicalDeviceProperties, PhysicalDeviceType,
};

use crate::{
    core::debug::errors::EngineError,
    debug, error,
    renderer::vulkan::{
        vulkan_types::VulkanRendererBackend,
        vulkan_utils::device_features::physical_device_features_to_vector,
    },
};

use super::{device_requirements::DeviceRequirements, queues::Queues};

#[derive(Default, Debug)]
pub(crate) struct PhysicalDeviceInfo {
    pub queues: Queues,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub extension_properties: Vec<ExtensionProperties>,
    pub memory_properties: PhysicalDeviceMemoryProperties,
    pub depth_format: Option<Format>,
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

    fn are_queue_families_requirements_fullfiled(
        requirements: &DeviceRequirements,
        device_info: &PhysicalDeviceInfo,
    ) -> bool {
        !(requirements.does_require_graphics_queue
            && device_info.queues.graphics_family_index.is_none()
            || requirements.does_require_present_queue
                && device_info.queues.present_family_index.is_none()
            || requirements.does_require_compute_queue
                && device_info.queues.graphics_family_index.is_none()
            || requirements.does_require_transfer_queue
                && device_info.queues.graphics_family_index.is_none())
    }

    fn are_swapchain_requirements_fullfiled(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<bool, EngineError> {
        Ok(self.query_swapchain_support(physical_device)?.is_complete())
    }

    fn are_extensions_requirements_fullfiled(
        &self,
        requirements: &DeviceRequirements,
        physical_device_info: &PhysicalDeviceInfo,
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
        physical_device_info: &PhysicalDeviceInfo,
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
                debug!(
                    "Device should support {:?}",
                    required_features_as_vec[feature].0
                );
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn physical_device_info_init(
        &self,
        physical_device: &PhysicalDevice,
    ) -> Result<PhysicalDeviceInfo, EngineError> {
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
            queues: Queues::default(),
            depth_format: None,
        })
    }

    fn is_device_suitable(
        &self,
        physical_device: &PhysicalDevice,
        requirements: &DeviceRequirements,
    ) -> Result<(bool, Option<PhysicalDeviceInfo>), EngineError> {
        let mut physical_device_info = self.physical_device_info_init(physical_device)?;
        physical_device_info.queues = self.queue_family_properties_create(physical_device)?;

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
        self.context.physical_device_info = None;
        self.context.physical_device = None;
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

    pub(crate) fn device_find_memory_index(
        &self,
        type_filter: u32,
        property_flags: MemoryPropertyFlags,
    ) -> Result<u32, EngineError> {
        let memory_properties = unsafe {
            let instance = self.get_instance()?;
            instance.get_physical_device_memory_properties(*self.get_physical_device()?)
        };

        for (index, memory_type) in memory_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << index) != 0)
                && memory_type.property_flags.intersects(property_flags)
            {
                return Ok(index as u32);
            }
        }

        error!("Unable to find suitable vulkan memory type");
        Err(EngineError::VulkanFailed)
    }

    pub(crate) fn device_detect_depth_format(&mut self) -> Result<(), EngineError> {
        // Format candidates
        let candidates = [
            Format::D32_SFLOAT,
            Format::D32_SFLOAT_S8_UINT,
            Format::D24_UNORM_S8_UINT,
        ];
        let flags = FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;
        let physical_device = self.context.physical_device.as_ref().unwrap();

        for candidate in candidates {
            let format_properties = unsafe {
                self.get_instance()?
                    .get_physical_device_format_properties(*physical_device, candidate)
            };
            if format_properties.linear_tiling_features.intersects(flags) {
                let device_info = self.context.physical_device_info.as_mut().unwrap();
                device_info.depth_format = Some(candidate);
                return Ok(());
            }
            if format_properties.optimal_tiling_features.intersects(flags) {
                let device_info = self.context.physical_device_info.as_mut().unwrap();
                device_info.depth_format = Some(candidate);
                return Ok(());
            }
        }

        error!("Failed to detect the vulkan physical device depth forma");
        Err(EngineError::VulkanFailed)
    }
}
