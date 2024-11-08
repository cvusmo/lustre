use gtk4::prelude::*;
use gtk4::DrawingArea;
use wgpu::Surface;
use std::rc::Rc;
use std::cell::RefCell;

// Create a struct to handle wgpu state
struct WgpuContext {
    instance: wgpu::Instance,
    surface: Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl WgpuContext {
    async fn new(width: i32, height: i32) -> Self {
        // Create a new instance of WGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Create a dummy window surface or use another method to create the WGPU surface

        // Request an adapter for the device
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None, // Add compatible surface if available
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Request a device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        // Get the surface configuration
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb, // Example format
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // surface.configure(&device, &config); // Configure surface if you have one

        Self {
            instance,
            surface: /* Placeholder for your surface */, // Set the surface properly
            device,
            queue,
            config,
        }
    }

    fn render(&mut self) {
        // Placeholder for rendering logic
        // You would use `self.surface` to get the frame and `self.device` to execute commands
    }
}

pub fn setup_wgpu_rendering(drawing_area: &DrawingArea) {
    // Connect the realize signal for the drawing area
    drawing_area.connect_realize(move |area| {
        // Clone drawing_area properties before passing to async function
        let width = area.allocated_width();
        let height = area.allocated_height();
        
        // Create a reference-counted WGPU context, wrapped in RefCell to allow interior mutability
        let wgpu_context = Rc::new(RefCell::new(None));

        // Create WGPU context asynchronously
        let context_clone = wgpu_context.clone();
        glib::MainContext::default().spawn_local(async move {
            let context = WgpuContext::new(width, height).await;
            *context_clone.borrow_mut() = Some(context);
        });

        // Set the draw function for rendering
        let context_clone = wgpu_context.clone();
        area.set_draw_func(move |_widget, _context, _width, _height| {
            if let Some(ref mut context) = *context_clone.borrow_mut() {
                context.render();
            }
        });
    });
}

