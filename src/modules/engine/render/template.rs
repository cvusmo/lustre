// src/modules/engine/render/template.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::DrawingArea;
use gdk4::Display;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::sync::GpuFuture;
use vulkano::VulkanLibrary;
use vulkano_win::VkSurfaceBuild;
use winit::platform::unix::WindowBuilderExtUnix;
use winit::window::WindowBuilder;

struct VulkanContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
}

impl VulkanContext {
    fn new(surface: Arc<Surface>, width: u32, height: u32) -> Self {
        // Create a Vulkan instance
        let library = VulkanLibrary::new().unwrap();
        let instance = Instance::new(library, InstanceExtensions::none(), None).unwrap();

        // Select a physical device
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .expect("No device available");

        // Find a queue family that supports graphics
        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .expect("Couldn't find a graphical queue family");

        // Create a logical device and a graphics queue
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::none()
            },
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();

        let queue = queues.next().unwrap();

        // Create the swapchain
        let (swapchain, _) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: 2,
                image_format: Some(surface.format().unwrap().0),
                image_extent: [width, height],
                image_usage: vulkano::image::ImageUsage::color_attachment(),
                composite_alpha: vulkano::swapchain::CompositeAlpha::Opaque,
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            device,
            queue,
            swapchain,
        }
    }

    fn render(&self) {
        // Rendering logic goes here
        // For simplicity, we are not implementing the full rendering loop here
        println!("Rendering frame...");
    }
}

pub fn setup_vulkan_rendering(drawing_area: &DrawingArea) {
    let width = drawing_area.content_width() as u32;
    let height = drawing_area.content_height() as u32;

    let vulkan_context = Rc::new(RefCell::new(None));

    let context_clone = vulkan_context.clone();
    drawing_area.connect_realize(move |area| {
        let display = Display::default().expect("Failed to get default display");
        let gdk_window = area.window().expect("Failed to get GDK window");
        let window_handle = gdk_window.raw_handle();

        // Create a Vulkan surface using winit and vulkano-win
        let surface = WindowBuilder::new()
            .with_name("Vulkan Surface")
            .with_visible(false)
            .with_override_redirect(true)
            .build_vk_surface(&window_handle, display.clone())
            .unwrap();

        let context = VulkanContext::new(surface, width, height);
        *context_clone.borrow_mut() = Some(context);
    });

    let context_clone = vulkan_context.clone();
    drawing_area.set_draw_func(move |_area, _cairo_context, _width, _height| {
        if let Some(ref context) = *context_clone.borrow() {
            context.render();
        }
    });
}

