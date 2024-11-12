// src/modules/render/vulkan/event_handler.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_info, AppState};
use crate::modules::render::vulkan::wayland::swapchain_handler::create_swapchain;
use crate::modules::render::vulkan::wayland::vulkan_surface::create_vulkan_surface;

use std::sync::{Arc, Mutex};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::Version;
use vulkano::VulkanLibrary;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Window>,
}

// ApplicationHandler
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                    // Placeholder: Redraw or Vulkan rendering code can go here.
                }
            }
            _ => (),
        }
    }
}

// Function to run event loop
pub fn run_event_loop(state: &Arc<Mutex<AppState>>) {
    log_info(state, "Searching Vulkan library...");
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");

    let instance = Instance::new(
        library.clone(),
        InstanceCreateInfo {
            application_name: Some("GameEngine".to_string()),
            application_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            engine_name: Some("Engine".to_string()),
            engine_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            enabled_extensions: InstanceExtensions {
                khr_surface: true,
                khr_wayland_surface: true,
                ..InstanceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("Failed to create Vulkan instance");

    log_info(state, "Selecting physical device...");
    let physical_device = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No physical device available");

    log_info(
        state,
        &format!(
            "Selected physical device: {}",
            physical_device.properties().device_name
        ),
    );

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (device, mut queues) = Device::new(
        Arc::new(physical_device),
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            queue_create_infos: vec![QueueCreateInfo::default()],
            ..Default::default()
        },
    )
    .expect("Failed to create device");

    let queue = queues.next().expect("Failed to get queue");

    // Initialize event loop and app
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App::default();

    log_info(state, "Running event loop...");
    event_loop.run(move |event, event_loop| {
        match event {
            winit::event::Event::Resumed => app.resumed(event_loop),
            winit::event::Event::WindowEvent { window_id, event } => {
                app.window_event(event_loop, window_id, event)
            }
            _ => (),
        }

        if let Some(window) = app.window.as_ref() {
            let surface = create_vulkan_surface(instance.clone(), window);
            let (swapchain, images) =
                create_swapchain(device.clone(), surface.clone(), queue.clone());

            // Use `swapchain` and `images` for rendering
        }
    });
}
