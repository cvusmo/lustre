// src/modules/engine/render/template.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::DrawingArea;
use wgpu::Surface;
use std::rc::Rc;
use std::cell::RefCell;

struct WgpuContext<'a> {
    instance: wgpu::Instance,
    surface: Option<Surface<'a>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl<'a> WgpuContext<'a> {
    async fn new(width: i32, height: i32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Placeholder for creating the WGPU surface

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

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

        Self {
            instance,
            surface: None, // Configure this properly later
            device,
            queue,
            config,
        }
    }

    fn render(&mut self) {
        // Placeholder for rendering logic
    }
}

pub fn setup_wgpu_rendering(drawing_area: &DrawingArea) {
    drawing_area.connect_realize(move |area| {
        let width = area.allocated_width();
        let height = area.allocated_height();

        let wgpu_context = Rc::new(RefCell::new(None));

        let context_clone = wgpu_context.clone();
        glib::MainContext::default().spawn_local(async move {
            let context = WgpuContext::new(width, height).await;
            *context_clone.borrow_mut() = Some(context);
        });

        let context_clone = wgpu_context.clone();
        area.set_draw_func(move |_widget, _context, _width, _height| {
            if let Some(ref mut context) = *context_clone.borrow_mut() {
                context.render();
            }
        });
    });
}

