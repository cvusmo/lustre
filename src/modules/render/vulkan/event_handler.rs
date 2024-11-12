// src/modules/render/vulkan_event_handler.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};
use crate::modules::render::vulkan::swapchain_handler::create_swapchain;
use crate::modules::render::vulkan::vulkan_surface::create_vulkan_surface;

use std::sync::{Arc, Mutex};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::Version;
use vulkano::VulkanLibrary;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
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

    log_info(state, "Initializing Winit event loop...");
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut app = App::default();
    let event_loop_ref = &event_loop;

    let device = Device::new(
        instance.clone(),
        DeviceCreateInfo {
            enabled_extensions: DeviceExtensions::khr_swapchain,
            ..Default::default()
        },
    )
    .expect("Failed to create device");

    if let Some(window) = app.window.as_ref() {
        let surface = create_vulkan_surface(instance.clone(), window);
        let (swapchain, images) = create_swapchain(device.clone(), surface.clone());

        // You can now use `swapchain` and `images` for rendering
    }

    log_info(state, "Running event loop...");
    event_loop_ref.set_control_flow(ControlFlow::Wait);

    if let Err(e) = event_loop_ref.run_app(&mut app) {
        log_error(state, &format!("Event loop terminated with error: {}", e));
    } else {
        log_info(state, "Event loop has exited successfully.");
    }
}
