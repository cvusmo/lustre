// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre
// src/engine/render.rs

use crate::engine::core::voxel::generate_voxel_mesh;
use crate::shaders::{fs, object_fs, object_vs, vs};
use crate::state::AppState;

use nalgebra::{Matrix4, Perspective3, Point3, Rotation3, Vector3};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo,
};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo,
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
use vulkano::pipeline::layout::{PipelineLayoutCreateInfo, PushConstantRange};
use vulkano::pipeline::{
    GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::shader::ShaderStages;
use vulkano::swapchain::{
    PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct MainVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],
}

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct ObjectVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
    #[format(R32G32_SFLOAT)]
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(BufferContents, Clone, Copy)]
struct PushConstants {
    mvp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

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

fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(),
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

fn get_terrain_pipeline(
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

    let push_constant_range = PushConstantRange {
        stages: ShaderStages::VERTEX,
        offset: 0,
        size: std::mem::size_of::<PushConstants>() as u32,
    };

    let pipeline_layout_create_info = PipelineLayoutCreateInfo {
        push_constant_ranges: vec![push_constant_range],
        ..Default::default()
    };

    let layout = PipelineLayout::new(device.clone(), pipeline_layout_create_info)
        .expect("failed to create terrain pipeline layout");

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

fn get_object_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
) -> Arc<GraphicsPipeline> {
    let vs_entry = vs.entry_point("main").unwrap();
    let fs_entry = fs.entry_point("main").unwrap();

    let vertex_input_state = ObjectVertex::per_vertex().definition(&vs_entry).unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs_entry),
        PipelineShaderStageCreateInfo::new(fs_entry),
    ];

    let push_constant_range = PushConstantRange {
        stages: ShaderStages::VERTEX,
        offset: 0,
        size: std::mem::size_of::<PushConstants>() as u32,
    };

    let pipeline_layout_create_info = PipelineLayoutCreateInfo {
        push_constant_ranges: vec![push_constant_range],
        ..Default::default()
    };

    let layout = PipelineLayout::new(device.clone(), pipeline_layout_create_info)
        .expect("failed to create object pipeline layout");

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

fn get_command_buffers(
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    queue: &Arc<Queue>,
    terrain_pipeline: Arc<GraphicsPipeline>,
    object_pipeline: Arc<GraphicsPipeline>,
    framebuffers: &[Arc<Framebuffer>],
    terrain_vertex_buffer: &Subbuffer<[MainVertex]>,
    terrain_index_buffer: &Subbuffer<[u32]>,
    object_vertex_buffer: &Subbuffer<[ObjectVertex]>,
    object_index_buffer: &Subbuffer<[u32]>,
    push_constants: PushConstants,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator.clone(),
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            )
            .unwrap();

            let mut render_pass_info = RenderPassBeginInfo::framebuffer(framebuffer.clone());
            render_pass_info.clear_values = vec![Some([0.2, 0.2, 0.2, 1.0].into())];

            let subpass_info = SubpassBeginInfo::default();

            let terrain_index_count =
                terrain_index_buffer.size() as u32 / std::mem::size_of::<u32>() as u32;
            let object_index_count =
                object_index_buffer.size() as u32 / std::mem::size_of::<u32>() as u32;

            unsafe {
                builder
                    .begin_render_pass(render_pass_info, subpass_info)
                    .unwrap();

                if terrain_index_count > 1 {
                    builder
                        .bind_pipeline_graphics(terrain_pipeline.clone())
                        .expect("failed to bind terrain pipeline")
                        .push_constants(terrain_pipeline.layout().clone(), 0, push_constants)
                        .unwrap()
                        .bind_vertex_buffers(0, terrain_vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(terrain_index_buffer.clone())
                        .unwrap()
                        .draw_indexed(terrain_index_count, 1, 0, 0, 0)
                        .unwrap();
                }

                if object_index_count > 1 {
                    builder
                        .bind_pipeline_graphics(object_pipeline.clone())
                        .expect("failed to bind object pipeline")
                        .push_constants(object_pipeline.layout().clone(), 0, push_constants)
                        .unwrap()
                        .bind_vertex_buffers(0, object_vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(object_index_buffer.clone())
                        .unwrap()
                        .draw_indexed(object_index_count, 1, 0, 0, 0)
                        .unwrap();
                }

                builder.end_render_pass(Default::default()).unwrap();
            }
            builder.build().unwrap()
        })
        .collect::<Vec<_>>()
}

pub fn lustre_render(instance: Arc<Instance>, surface: Arc<Surface>, state: Arc<Mutex<AppState>>) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, _) = get_physical_device(&instance, &surface, &device_extensions);

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("couldn't find a graphics queue family") as u32;

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

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let (format, _colorspace) = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[1];

    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");
    let image_extent = caps.current_extent.unwrap_or([1025, 1024]);

    let (swapchain, swapchain_images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_format: format,
            image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            present_mode: PresentMode::Immediate,
            ..Default::default()
        },
    )
    .expect("failed to create swapchain");

    let render_pass = get_render_pass(device.clone(), swapchain.clone());
    let framebuffers = get_framebuffers(&swapchain_images, render_pass.clone());

    let (image_index, _suboptimal, acquire_future) =
        vulkano::swapchain::acquire_next_image(swapchain.clone(), None)
            .expect("failed to acquire next image");

    let elapsed = Instant::now()
        .duration_since(state.lock().unwrap().start_time)
        .as_secs_f32();
    let angle = elapsed * 45.0_f32.to_radians();

    let aspect_ratio = image_extent[0] as f32 / image_extent[1] as f32;
    let proj = Perspective3::new(aspect_ratio, 75.0_f32.to_radians(), 0.1, 1000.0);

    let matrix_view = Matrix4::look_at_rh(
        &Point3::new(96.0, 96.0, 96.0),
        &Point3::new(32.0, 32.0, 32.0),
        &Vector3::y(),
    );

    let model = Rotation3::from_axis_angle(&Vector3::y_axis(), angle).to_homogeneous();
    let final_model: [[f32; 4]; 4] = model.into();
    let mvp = proj.to_homogeneous() * matrix_view * model;
    let mvp_final: [[f32; 4]; 4] = mvp.into();

    let push_constants = PushConstants {
        mvp: mvp_final,
        model: final_model,
    };

    let state_guard = state.lock().unwrap();
    let (terrain_vertices, terrain_indices) = generate_voxel_mesh(&state_guard.voxel_grid);
    //println!(
    //"Terrain Vertices: {}, Indices: {}",
    //terrain_vertices.len(),
    //terrain_indices.len()
    //);

    let terrain_vertex_buffer = if !terrain_vertices.is_empty() {
        Buffer::from_iter(
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
            terrain_vertices.into_iter(),
        )
        .expect("Failed to create terrain vertex buffer")
    } else {
        Buffer::from_iter(
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
            vec![MainVertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
                color: [0.0, 0.0, 0.0],
            }]
            .into_iter(),
        )
        .expect("Failed to create placeholder terrain vertex buffer")
    };

    let terrain_index_buffer = if !terrain_indices.is_empty() {
        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            terrain_indices.into_iter(),
        )
        .expect("Failed to create terrain index buffer")
    } else {
        Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vec![0].into_iter(),
        )
        .expect("Failed to create placeholder terrain index buffer")
    };

    let (object_vertices, object_indices) =
        crate::engine::core::objects::load_mesh("placeholder.glb")
            .expect("Failed to load object mesh");
    //println!(
    //"Object Vertices: {}, Indices: {}",
    //object_vertices.len(),
    //object_indices.len()
    //);

    let object_vertex_buffer = Buffer::from_iter(
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
        object_vertices.into_iter(),
    )
    .expect("Failed to create object vertex buffer");

    let object_index_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        object_indices.into_iter(),
    )
    .expect("Failed to create object index buffer");

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [image_extent[0] as f32, image_extent[1] as f32],
        depth_range: 0.0..=1.0,
    };

    let terrain_vs_module = vs::load(device.clone()).expect("failed to load terrain vertex shader");
    let terrain_fs_module =
        fs::load(device.clone()).expect("failed to load terrain fragment shader");
    let object_vs_module =
        object_vs::load(device.clone()).expect("failed to load object vertex shader");
    let object_fs_module =
        object_fs::load(device.clone()).expect("failed to load object fragment shader");

    let terrain_pipeline = get_terrain_pipeline(
        device.clone(),
        terrain_vs_module,
        terrain_fs_module,
        render_pass.clone(),
        viewport.clone(),
    );

    let object_pipeline = get_object_pipeline(
        device.clone(),
        object_vs_module,
        object_fs_module,
        render_pass.clone(),
        viewport,
    );

    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        StandardCommandBufferAllocatorCreateInfo::default(),
    ));

    let command_buffers = get_command_buffers(
        command_buffer_allocator,
        &queue,
        terrain_pipeline,
        object_pipeline,
        &framebuffers,
        &terrain_vertex_buffer,
        &terrain_index_buffer,
        &object_vertex_buffer,
        &object_index_buffer,
        push_constants,
    );

    let command_buffer = command_buffers[image_index as usize].clone();

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
}
