// src/modules/engine/render/core.rs

use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, PresentMode, CompositeAlpha};
use vulkano::VulkanLibrary;
use vulkano::image::ImageUsage;
use std::sync::Arc;
use winit::window::Window;

pub struct VulkanContext {
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
}

impl VulkanContext {
    /// Initialize VulkanContext with basic Vulkan setup
    pub fn new(window: &Window, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        // Step 1: Create Vulkan instance
        let library = VulkanLibrary::new()?;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: vulkano::instance::InstanceExtensions {
                    khr_surface: true,
                    ..vulkano::instance::InstanceExtensions::empty()
                },
                ..Default::default()
            },
        )?;

        // Step 2: Create surface for the window
        let surface = unsafe { vulkano_win::create_surface_from_winit(window, instance.clone())? };

        // Step 3: Select physical device (GPU)
        let physical_device = instance
            .enumerate_physical_devices()?
            .next()
            .ok_or("No suitable physical device found")?;

        // Step 4: Create logical device and queue
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|q| q.queue_flags.graphics && physical_device.surface_support(q.id() as u32, &surface).unwrap_or(false))
            .ok_or("No suitable queue family found")?;
        
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: DeviceExtensions { khr_swapchain: true, ..DeviceExtensions::empty() },
                queue_create_infos: vec![QueueCreateInfo::family_index(queue_family_index)],
                ..Default::default()
            },
        )?;
        
        let queue = queues.next().ok_or("No queue created")?;

        // Step 5: Create swapchain
        let caps = physical_device.surface_capabilities(&surface, Default::default())?;
        let format = physical_device.surface_formats(&surface, Default::default())?.first().unwrap().0;
        let extent = caps.current_extent.unwrap_or([width, height]);
        let (swapchain, _images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count,
                image_format: Some(format),
                image_extent: extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: CompositeAlpha::Opaque,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        )?;

        Ok(Self {
            instance,
            device,
            queue,
            swapchain,
        })
    }

    /// Render function placeholder for drawing frames
    pub fn render(&self) {
        // In the future, frame rendering logic goes here
        println!("Rendering a frame...");
    }

    /// Resizes the swapchain when the window size changes
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        println!("Resizing to: {}x{}", new_width, new_height);
        // Handle resize logic here, such as recreating the swapchain with the new dimensions
    }

    /// Clean up Vulkan resources
    pub fn cleanup(&self) {
        // In the future, Vulkan cleanup logic goes here
        println!("Cleaning up Vulkan resources...");
    }
}

