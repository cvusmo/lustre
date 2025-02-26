// Copyright 2024 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/window.rs

use crate::engine::render::lustre_render;
use crate::state::AppState;
use std::sync::{Arc, Mutex};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::library::VulkanLibrary;
use vulkano::swapchain::Surface;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    surface: Option<Arc<Surface>>,
    instance: Option<Arc<Instance>>,
    state: Option<Arc<Mutex<AppState>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window.
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window");
        let window = Arc::new(window);
        self.window = Some(window.clone());

        // Create the Vulkan instance.
        let library = VulkanLibrary::new().expect("Failed to load Vulkan library");
        let set_display_handle = window.display_handle().unwrap();
        let required_extensions = Surface::required_extensions(&set_display_handle)
            .expect("Failed to get required surface extensions");
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .expect("Failed to create instance");
        self.instance = Some(instance.clone());

        // Create the surface.
        let surface = Surface::from_window(instance, window).expect("Failed to create surface");
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Closing");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(ref instance), Some(ref surface), Some(ref state)) =
                    (&self.instance, &self.surface, &self.state)
                {
                    lustre_render(instance.clone(), surface.clone(), state.clone());
                }
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }
}

pub fn lustre_window(state: Arc<Mutex<AppState>>) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    app.state = Some(state);
    event_loop.run_app(&mut app);
}
