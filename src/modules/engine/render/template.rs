use gtk4 as gtk;
use gtk::prelude::*;
use gtk::DrawingArea;
use wgpu::util::DeviceExt;
use std::sync::{Arc, Mutex};
use winit::platform::unix::WindowBuilderExtUnix;

// Create a struct to handle wgpu state
struct WgpuContext {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl WgpuContext {
    async fn new() -> Self {
        // Create a new instance of WGPU and configure it
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // Create a dummy surface configuration for now
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let surface = instance.create_surface(&winit::window::Window::new().unwrap());

        surface.configure(&device, &config);

        Self {
            instance,
            surface,
            device,
            queue,
            config,
        }
    }

    fn render(&mut self) {
        // Rendering function for WGPU
    }
}

pub fn setup_wgpu_rendering(drawing_area: &DrawingArea) {
    drawing_area.connect_realize(move |area| {
        // Integrate WGPU rendering when the widget is realized
        // Create WGPU context asynchronously
        let context = WgpuContext::new();

        glib::MainContext::default().spawn_local(async move {
            let mut wgpu_context = context.await;
            wgpu_context.render();
        });
    });
}

