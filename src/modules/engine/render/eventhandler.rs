// src/modules/render/eventhandler.rs
// github.com/cvusmo/gameengine

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::modules::engine::configuration::logger::{log_info, AppState};
use std::sync::{Arc, Mutex};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::swapchain::Surface;
use vulkano::VulkanLibrary;

#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window when the application resumes
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
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

pub fn run_event_loop(state: &Arc<Mutex<AppState>>) {
    // Create Vulkan instance
    log_info(state, "Creating Vulkan instance...");
    let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: InstanceExtensions {
                khr_surface: true,
                khr_wayland_surface: true, // or khr_xcb_surface for X11
                ..InstanceExtensions::empty()
            },
            ..Default::default()
        },
    )
    .expect("Failed to create Vulkan instance");

    // Initialize the Winit event loop and application
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();

    // Create the ActiveEventLoop and start the event loop
    log_info(state, "Running event loop...");
    event_loop.set_control_flow(ControlFlow::Poll); // For games or continuous rendering
    event_loop.run_app(&mut app);
}
