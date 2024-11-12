// src/modules/engine/render/vulkan/wayland/core.rs

use std::sync::{Arc, Mutex};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateFlags, QueueCreateInfo,
    QueueFlags,
};
use vulkano::image::ImageUsage;
use vulkano::instance::Instance;
use vulkano::swapchain::{
    CompositeAlpha, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo,
};

use crate::modules::engine::configuration::logger::{log_info, AppState};

pub struct VulkanContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<vulkano::image::Image>>,
}

// VulkanContext
impl VulkanContext {
    pub fn new(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        width: u32,
        height: u32,
        state: &Arc<Mutex<AppState>>,
    ) -> Self {
        // Enumerate the physical devices and pick one based on user preferences.
        let physical_device = instance
            .enumerate_physical_devices()
            .unwrap_or_else(|err| panic!("Couldn't enumerate physical devices: {:?}", err))
            .next()
            .expect("No physical device");

        // Log information about the physical device
        log_info(
            state,
            &format!(
                "Using device: {} (type: {:?})",
                physical_device.properties().device_name,
                physical_device.properties().device_type
            ),
        );

        // Specify features and extensions required for the device
        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let required_features = Features::empty();

        // Ensure that the physical device supports the required extensions
        let supported_extensions = physical_device.supported_extensions();
        if !supported_extensions.contains(&required_extensions) {
            panic!("The physical device does not support the required extensions.");
        }

        // Find a queue family that supports graphics and presentation
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(index, q)| {
                q.queue_flags.contains(QueueFlags::GRAPHICS)
                    && physical_device
                        .surface_support(*index as u32, &surface)
                        .unwrap_or(false)
            })
            .map(|(index, _)| index as u32)
            .expect("Couldn't find a suitable queue family that supports graphics.");

        // Create the device and the queue
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    queues: vec![0.5],                // Priority of the queue
                    flags: QueueCreateFlags::empty(), // Sets to empty
                    ..Default::default()
                }],
                enabled_extensions: required_extensions,
                enabled_features: required_features,
                physical_devices: vec![physical_device.clone()].into(),
                private_data_slot_request_count: 0,
                ..Default::default()
            },
        )
        .expect("Failed to create logical device");

        let queue = queues.next().unwrap();

        // Get the surface capabilities
        let surface_capabilities = physical_device
            .surface_capabilities(&surface, SurfaceInfo::default())
            .expect("Failed to get surface capabilities");

        // Determine the image extent to use
        let image_extent = surface_capabilities
            .current_extent
            .unwrap_or([width, height]);

        // Use double-buffering if possible.
        let min_image_count = match surface_capabilities.max_image_count {
            None => std::cmp::max(2, surface_capabilities.min_image_count),
            Some(limit) => std::cmp::min(
                std::cmp::max(2, surface_capabilities.min_image_count),
                limit,
            ),
        };

        // Preserve the current surface transform
        let pre_transform = surface_capabilities.current_transform;

        // Use the first available format
        let (image_format, _) = physical_device
            .surface_formats(&surface, SurfaceInfo::default())
            .expect("Failed to get surface formats")[0];

        // Create the swapchain for rendering to the window surface
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: CompositeAlpha::Opaque,
                present_mode: PresentMode::Fifo,
                pre_transform,
                ..Default::default()
            },
        )
        .expect("Failed to create swapchain");

        Self {
            device,
            queue,
            swapchain,
            images,
        }
    }

    /// Render function placeholder for drawing frames
    pub fn render(&self, state: &Arc<Mutex<AppState>>) {
        // In the future, frame rendering logic goes here
        log_info(state, "Rendering a frame...");
    }

    /// Resizes the swapchain when the window size changes
    pub fn resize(&mut self, _new_width: u32, _new_height: u32, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Resizing to: {}x{}");
        // Handle resize logic here, such as recreating the swapchain with the new dimensions
    }

    /// Clean up Vulkan resources
    pub fn cleanup(&self, state: &Arc<Mutex<AppState>>) {
        // In the future, Vulkan cleanup logic goes here
        log_info(state, "Cleaning up Vulkan resources...");
    }
}
