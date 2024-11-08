// src/modules/engine/render/template.rs
// github.com/cvusmo/gameengine

use gtk4::prelude::*;
use gtk4::DrawingArea;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, WindowHandle};
use wgpu::{Instance, Surface, SurfaceTarget, MemoryHints};
use std::rc::Rc;
use std::cell::RefCell;

struct WgpuContext<'a> {
    instance: wgpu::Instance,
    surface: Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl<'a> WgpuContext<'a> {
    async fn new(width: i32, height: i32, surface: Surface<'a>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
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
                    memory_hints: MemoryHints::Performance, // Set to prioritize performance
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0],
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // Set a default frame latency
        };

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
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire next surface texture: {e}");
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

pub fn setup_wgpu_rendering(drawing_area: &DrawingArea) {
    drawing_area.connect_realize(move |area| {
        let width = area.allocated_width();
        let height = area.allocated_height();

        let wgpu_context = Rc::new(RefCell::new(None));

        let context_clone = wgpu_context.clone();
        glib::MainContext::default().spawn_local(async move {
            // Get the toplevel window associated with the DrawingArea
            if let Some(window) = area.native() {
                if let Ok(window) = window.downcast::<gtk4::Window>() {
                    let raw_window_handle = window.raw_window_handle();

                    // Create the WGPU instance
                    let instance = Instance::new(wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::PRIMARY,
                        ..Default::default()
                    });

                    // Create the WGPU surface using SurfaceTarget::Window
                    let surface_target = SurfaceTarget::Window(Box::new(raw_window_handle));
                    let surface = match instance.create_surface(surface_target) {
                        Ok(surface) => surface,
                        Err(e) => {
                            eprintln!("Failed to create WGPU surface: {e}");
                            return;
                        }
                    };

                    let context = WgpuContext::new(width, height, surface).await;
                    *context_clone.borrow_mut() = Some(context);
                } else {
                    eprintln!("Failed to downcast native window to gtk4::Window.");
                }
            } else {
                eprintln!("Failed to get toplevel window for creating WGPU surface.");
            }
        });

        let context_clone = wgpu_context.clone();
        area.set_draw_func(move |_widget, _context, _width, _height| {
            if let Some(ref mut context) = *context_clone.borrow_mut() {
                context.render();
            }
        });
    });
}

