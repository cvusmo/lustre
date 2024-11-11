// src/modules/render/eventhandler.rs
// github.com/cvusmo/gameengine

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};

use std::sync::{Arc, Mutex};
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
        // Create the window when the application resumes
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
    // Create Vulkan instance
    log_info(state, "Searching Vulkan library...");
    let library = match VulkanLibrary::new() {
        Ok(lib) => lib,
        Err(e) => {
            log_error(state, &format!("Failed to load Vulkan library: {}", e));
            return;
        }
    };

    log_info(
        state,
        "Creating Vulkan instance with required extensions: khr_surface, khr_wayland_surface...",
    );
    let _instance = match Instance::new(
        library,
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
    ) {
        Ok(inst) => inst,
        Err(e) => {
            log_error(
                state,
                &format!(
                    "Failed to create Vulkan instance: a validation error occurred - {}",
                    e
                ),
            );
            return;
        }
    };

    // Initialize the Winit event loop and application
    log_info(state, "Initializing Winit event loop...");
    let event_loop = match EventLoop::new() {
        Ok(loop_instance) => loop_instance,
        Err(e) => {
            log_error(state, &format!("Failed to create event loop: {}", e));
            return;
        }
    };

    let mut app = App::default();

    // Create the ActiveEventLoop and start the event loop
    log_info(state, "Running event loop...");
    event_loop.set_control_flow(ControlFlow::Poll);

    // Handle potential error from running event loop, assuming run_app might fail.
    if let Err(e) = event_loop.run_app(&mut app) {
        log_error(state, &format!("Event loop terminated with error: {}", e));
    } else {
        log_info(state, "Event loop has exited successfully.");
    }
}

