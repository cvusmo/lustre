// src/modules/engine/render/template.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea};
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateFlags, QueueCreateInfo,
    QueueFlags,
};
use vulkano::image::ImageUsage;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::swapchain::{
    CompositeAlpha, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo,
};
use vulkano::VulkanLibrary;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};

struct VulkanContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<vulkano::image::Image>>,
}

// VulkanContext
impl VulkanContext {
    fn new(
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

    fn render(&self, state: &Arc<Mutex<AppState>>) {
        // Placeholder for rendering logic
        log_info(state, "Rendering frame...");
    }
}

// Function for Vulkan Rendering
pub fn setup_vulkan_rendering(state: &Arc<Mutex<AppState>>) {
    log_info(state, "Setting up Vulkan rendering...");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vulkan Window")
        .build(&event_loop)
        .expect("Failed to create window");

    // Create a Vulkan instance
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: InstanceExtensions {
                khr_surface: true,
                ext_debug_utils: true,
                ..InstanceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("Failed to create Vulkan instance");

    // Create a Vulkan surface from the winit window
    let surface = unsafe {
        vulkano_win::create_surface_from_winit(&window, instance.clone())
            .expect("Failed to create Vulkan surface")
    };

    let width = 800;
    let height = 600;

    let vulkan_context = VulkanContext::new(
        Arc::clone(&instance),
        Arc::new(surface),
        width,
        height,
        state,
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(physical_size) => {
                    println!(
                        "Window resized: {}x{}",
                        physical_size.width, physical_size.height
                    );
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                vulkan_context.render(state);
            }
            _ => (),
        }
    });
}
