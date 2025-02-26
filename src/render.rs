// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/render.rs

use std::sync::Arc;

#[warn(unused_imports)]
use crate::shaders::fs;
use crate::shaders::vs;
use crate::state::log_info;

use image::{ImageBuffer, Rgba};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo, PrimaryAutoCommandBuffer,
    RenderPassBeginInfo, SubpassBeginInfo,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::Instance;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{
    PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::{self, GpuFuture};

// Create Vertices
#[derive(BufferContents, Vertex)]
#[repr(C)]
struct MainVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

// Get Physical Device
pub fn get_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("failed to enumerate physical devices")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available")
}

// Get Render Pass
fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(), // Use format as swapchain
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap()
}

// Get Framebuffers
fn get_framebuffers(
    swapchain_images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    swapchain_images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

// Get graphic pipeline
fn get_graphic_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs_entry = vs.entry_point("main").unwrap();
    let fs_entry = fs.entry_point("main").unwrap();

    let vertex_input_state = MainVertex::per_vertex().definition(&vs_entry).unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

// Get command buffers
fn get_command_buffers(
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: &Arc<Queue>,
    graphic_pipeline: &Arc<GraphicsPipeline>,
    framebuffer: &[Arc<Framebuffer>],
    vertex_buffer: &Subbuffer<[MainVertex]>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffer
        .iter()
        .map(|framebuffer| {
            // Create a new command buffer builder for each framebuffer.
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            let mut render_pass_info = RenderPassBeginInfo::framebuffer(framebuffer.clone());
            render_pass_info.clear_values = vec![Some([0.0, 0.0, 1.0, 1.0].into())];

            let subpass_info = SubpassBeginInfo::default();

            // Record commands:
            unsafe {
                builder
                    .begin_render_pass(render_pass_info, subpass_info)
                    .unwrap()
                    .bind_pipeline_graphics(graphic_pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .unwrap()
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    .end_render_pass(Default::default())
                    .unwrap();
            }
            builder.build().unwrap()
        })
        .collect::<Vec<_>>()
}

// Render function
pub fn lustre_render(instance: Arc<Instance>, surface: Arc<Surface>) {
    // Define required device extensions.
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    // Select a physical device.
    let (physical_device, _) = get_physical_device(&instance, &surface, &device_extensions);

    log_info("Physical device is: {},");

    // Choose a graphics queue family.
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("couldn't find a graphics queue family") as u32;

    // Create the logical device and retrieve the queue.
    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )
    .expect("failed to create device");
    let queue = queues.next().unwrap();

    // Create a memory allocator.
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    // Create the buffer information
    let buf = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    // Create the swapchain.
    let (format, _colorspace) = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0];

    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    let image_extent = caps.current_extent.unwrap_or([1024, 1024]);

    let (swapchain, swapchain_images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_format: format, // Store this format to use later
            image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            present_mode: PresentMode::Fifo,
            ..Default::default()
        },
    )
    .expect("failed to create swapchain");

    let vertex1 = MainVertex {
        position: [-0.5, -0.5],
    };

    let vertex2 = MainVertex {
        position: [0.0, 0.5],
    };

    let vertex3 = MainVertex {
        position: [0.5, -0.25],
    };

    // Triangle
    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    // let images = vec![swapchain_images.clone()];

    // Acquire swapchain image and present it
    let (image_index, suboptimal, acquire_future) =
        vulkano::swapchain::acquire_next_image(swapchain.clone(), None)
            .expect("failed to acquire next image.");

    // Single Render Pass && Swapchain Creation
    let render_pass = get_render_pass(device.clone(), swapchain.clone());

    // Creating Framebuffers
    let framebuffer = get_framebuffers(&swapchain_images, render_pass.clone());

    // Create viewport
    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [1024.0, 1024.0],
        depth_range: 0.0..=1.0,
    };

    let vs_module = vs::load(device.clone()).expect("failed to load vertex shader.");
    let fs_module = fs::load(device.clone()).expect("failed to laod fragment shader.");

    // Create pipeline
    let graphic_pipeline = get_graphic_pipeline(
        device.clone(),
        vs_module,
        fs_module,
        render_pass.clone(),
        viewport,
    );

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));

    // Get command buffers
    let command_buffers = get_command_buffers(
        command_buffer_allocator,
        &queue,
        &graphic_pipeline,
        &framebuffer,
        &vertex_buffer,
    );

    let command_buffer = command_buffers[0].clone();

    // Submit the command buffer.
    //let future = sync::now(device)
    let future = acquire_future
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(
            queue.clone(),
            SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
        )
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    // Generate image
    let content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &content[..]).unwrap();
    image.save("image.png").unwrap();

    println!("Everything succeeded!");
}
