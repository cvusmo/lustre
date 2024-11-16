// Copyright 2025 Nicholas Jordan. All Rights Reserved.
// github.com/cvusmo/lustre

// src/modules/engine/render/vulkan/wayland/core.rs

use std::sync::{Arc, Mutex};
// use std::time::Duration;
use std::collections::HashMap;
use vulkano::command_buffer::pool::{CommandPool, CommandPoolCreateInfo};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents,
};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateFlags, QueueCreateInfo,
    QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::ImageUsage;
use vulkano::instance::Instance;
use vulkano::pipeline::DynamicState;
use vulkano::pipeline::graphics::{
    color_blend::ColorBlendState,
    input_assembly::InputAssemblyState,
    rasterization::RasterizationState,
    tessellation::TessellationDomainOrigin,
    tessellation::TessellationState,
    vertex_input::{Vertex, VertexBufferDescription, VertexMemberInfo, VertexInputRate, VertexInputState},
    viewport::{Viewport, ViewportState},
    GraphicsPipeline, GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::layout::PipelineLayout;
use vulkano::pipeline::PipelineCreateFlags;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::swapchain;
use vulkano::swapchain::{
    CompositeAlpha, PresentMode, Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo,
    SwapchainPresentInfo,
};
use vulkano::sync::{
    fence::Fence, semaphore::Semaphore, semaphore::SemaphoreCreateInfo, GpuFuture,
};
use vulkano::VulkanLibrary;

use crate::modules::engine::configuration::logger::{log_error, log_info, AppState};

// #[allow(dead_code)]
// VulkanContext struct
pub struct VulkanContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<vulkano::image::Image>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    command_pool: Arc<CommandPool>,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    image_available_semaphore: Arc<Semaphore>,
    render_finished_semaphore: Arc<Semaphore>,
    in_flight_fence: Arc<Fence>,
}

// VulkanContext
impl VulkanContext {
    pub fn new(
        surface: Arc<Surface>,
        width: u32,
        height: u32,
        state: &Arc<Mutex<AppState>>,
    ) -> Self {
        // Create library
        let library = VulkanLibrary::new()
            .unwrap_or_else(|err| panic!("Couldn't load Vulkan library: {:?}", err));

        #[warn(unused_variables)]
        // Create layers
        let _layers: Vec<_> = library
            .layer_properties()
            .unwrap_or_else(|err| panic!("Failed to retrieve Vulkan layer properties: {:?}", err))
            .filter(|l| l.name() == "gameengine_layer")
            .collect();

        // Create instance
        let instance = Arc::new(
            Instance::new(library, Default::default()).expect("Failed to create Vulkan instance"),
        );

        // Enumerate the physical devices and pick one based on user preferences.
        let physical_device = instance
            .enumerate_physical_devices()
            .unwrap_or_else(|err| panic!("Couldn't enumerate physical devices: {:?}", err))
            .find(|device| {
                // Check if the device has sufficient VRAM (4GB or more)
                let memory_properties = device.memory_properties();
                let vram_size = memory_properties
                    .memory_heaps
                    .iter()
                    .map(|heap| heap.size)
                    .max()
                    .unwrap_or(0);

                // Require at least 4GB of VRAM
                vram_size >= 4 * 1024 * 1024 * 1024
            })
            .expect("No suitable physical device found with at least 4GB of VRAM.");

        // Log information about the physical device
        log_info(
            state,
            &format!(
                "Using device: {} (type: {:?}, VRAM: {} GB)",
                physical_device.properties().device_name,
                physical_device.properties().device_type,
                physical_device
                    .memory_properties()
                    .memory_heaps
                    .iter()
                    .map(|heap| heap.size)
                    .max()
                    .unwrap_or(0)
                    / (1024 * 1024 * 1024) // Convert bytes to GB
            ),
        );

        // Specify features and extensions required for the device
        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let required_features = Features::empty();

        // Ensure that the physical device supports the required extensions
        let supported_extensions = physical_device.supported_extensions();
        if !supported_extensions.contains(&required_extensions) {
            panic!("The physical device does not support the required extensions.");
        }

        // Find a queue family that supports graphics and presentation
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(index, q)| {
                q.queue_flags.contains(QueueFlags::GRAPHICS)
                    && physical_device
                        .surface_support(*index as u32, &surface)
                        .unwrap_or(false)
            })
            .map(|(index, _)| index as u32)
            .expect("Couldn't find a suitable queue family that supports graphics.");

        // Create the device and the queue
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    queues: vec![0.5],                // Priority of the queue
                    flags: QueueCreateFlags::empty(), // Sets to empty
                    ..Default::default()
                }],
                enabled_extensions: required_extensions,
                enabled_features: required_features,
                physical_devices: vec![physical_device.clone()].into(),
                private_data_slot_request_count: 0,
                ..Default::default()
            },
        )
        .expect("Failed to create logical device");

        let queue = queues.next().unwrap();

        // Get the surface capabilities
        let surface_capabilities = physical_device
            .surface_capabilities(&surface, SurfaceInfo::default())
            .expect("Failed to get surface capabilities");

        // Determine the image extent to use
        let image_extent = surface_capabilities
            .current_extent
            .unwrap_or([width, height]);

        // Use double-buffering if possible.
        let min_image_count = match surface_capabilities.max_image_count {
            None => std::cmp::max(2, surface_capabilities.min_image_count),
            Some(limit) => std::cmp::min(
                std::cmp::max(2, surface_capabilities.min_image_count),
                limit,
            ),
        };

        // Preserve the current surface transform
        let pre_transform = surface_capabilities.current_transform;

        // Use the first available format
        let (image_format, _) = physical_device
            .surface_formats(&surface, SurfaceInfo::default())
            .expect("Failed to get surface formats")[0];

        // Create the swapchain for rendering to the window surface
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: CompositeAlpha::Opaque,
                present_mode: PresentMode::Fifo,
                pre_transform,
                ..Default::default()
            },
        )
        .expect("Failed to create swapchain");

        // Create render pass
        let render_pass = VulkanContext::create_render_pass(device.clone());

        // Create Framebuffers
        let framebuffers = VulkanContext::create_framebuffers(render_pass.clone(), &images);

        // Create Command Pool and Buffers
        let (command_pool, command_buffers) = VulkanContext::create_command_pool_and_buffers(
            device.clone(),
            queue_family_index,
            &framebuffers,
            render_pass.clone(),
        );

        // Create synchronization Objects
        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            VulkanContext::create_sync_objects(device.clone());

        Self {
            device,
            queue,
            swapchain,
            images,
            render_pass,
            framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    // Function to create render pass
    fn build_render_pass(device: Arc<Device>) -> Arc<RenderPass> {
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: Format::R8G8B8A8_UNORM, // Match swapchain image format
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap()
    }

    // Function to create framebuffers
    fn create_framebuffers(
        render_pass: Arc<RenderPass>,
        images: &[Arc<vulkano::image::Image>],
    ) -> Vec<Arc<Framebuffer>> {
        images
            .iter()
            .map(|image| {
                let view_create_info = ImageViewCreateInfo::from_image(image);
                let view = ImageView::new(image.clone(), view_create_info).unwrap();

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        extent: [32; 2],
                        layers: 1,
                        ..Default::default()
                    },
                )
                .unwrap()
            })
            .collect()
    }

    // Function to create Command Pool and Command Buffers
    fn create_command_pool_and_buffers(
        device: Arc<Device>,
        queue_family_index: u32,
        framebuffers: &[Arc<Framebuffer>],
        _render_pass: Arc<RenderPass>,
    ) -> (Arc<CommandPool>, Vec<Arc<PrimaryAutoCommandBuffer>>) {
        let command_pool = Arc::new(
            CommandPool::new(
                device.clone(),
                CommandPoolCreateInfo {
                    queue_family_index,
                    ..Default::default()
                },
            )
            .expect("Failed to create Command Pool."),
        );

        // #[warn(unused_mut)]
        let command_buffers = Vec::new();

        for _framebuffer in framebuffers {
            // framebuffer
            println!("Placeholder for framebuffer.");
        }

        (command_pool, command_buffers)
    }

    // Function to synchronize objects
    fn create_sync_objects(device: Arc<Device>) -> (Arc<Semaphore>, Arc<Semaphore>, Arc<Fence>) {
        let semaphore_create_info = SemaphoreCreateInfo::default();

        let image_available_semaphore =
            Semaphore::new(device.clone(), semaphore_create_info.clone())
                .expect("Failed to create image available semaphore");
        let render_finished_semaphore = Semaphore::new(device.clone(), semaphore_create_info)
            .expect("Failed to create render finished semaphore");
        let in_flight_fence = Fence::new(device.clone(), Default::default())
            .expect("Failed to create in-flight-fence");
        (
            Arc::new(image_available_semaphore),
            Arc::new(render_finished_semaphore),
            Arc::new(in_flight_fence),
        )
    }

    /// Render function
    pub fn render(&mut self, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Rendering a frame...");

        let (image_index, suboptimal, acquire_future) = match swapchain::acquire_next_image(
            self.swapchain.clone(),
            //Some(Duration::from_secs(1)),
            None,
        ) {
            Ok(result) => result,
            Err(err) => {
                log_info(state, &format!("Failed to acquire next image: {:?}", err));
                return;
            }
        };

        // Log if acquistion is suboptimal
        if suboptimal {
            log_info(state, "Swapchain acquistion is suboptimal.");
            self.recreate_swapchain(state);
            return;
        }

        // Begin Rendering commands
        // let command_buffer = &self.command_buffers[image_index as usize];
        let command_buffer = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap()
        .begin_render_pass(
            self.framebuffers[image_index].clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 0.0, 1.0].into()],
        )
        .unwrap()
        .bind_pipeline_graphics(self.pipeline.clone())
        .draw(3, 1, 0, 0) // Draw a single triangle
        .unwrap()
        .end_render_pass()
        .unwrap()
        .build()
        .unwrap();

        // Wait for previous frame to finish
        self.in_flight_fence
            .wait(None)
            .expect("Failed to wait for in-flight fence.");

        // Reset the fence for next frame
        self.in_flight_fence
            .reset()
            .expect("Failed to reset in-flight fence.");

        // Begin the GPU commands by synchronizing the acquire future
        let future = acquire_future
            .then_execute(self.queue.clone(), command_buffer.clone())
            .expect("Failed to executed comand buffer.")
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => match future.wait(None) {
                Ok(_) => {}
                Err(err) => {
                    log_error(state, &format!("Failed to wait on future: {:?}", err));
                    return;
                }
            },
            Err(err) => {
                log_error(state, &format!("Failed to flush future: {:?}", err));
            }
        }
    }

    // Function to recreate swapchain
    pub fn recreate_swapchain(&mut self, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Recreating swapchain...");

        // Get surface capabilities
        let surface_capabilities = self
            .device
            .physical_device()
            .surface_capabilities(&self.swapchain.surface(), SurfaceInfo::default())
            .expect("Failed to get surface capabilities.");

        // Determine the new image extent
        let image_extent = surface_capabilities.current_extent.unwrap_or([3280, 2160]);

        // Create a new swapchain
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent,
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(err) => {
                log_error(state, &format!("Failed to recreate swapchain {:?}", err));
                return;
            }
        };

        // Update new swapchain and images
        self.swapchain = new_swapchain;
        self.images = new_images;

        // Recreate framebuffers
        self.framebuffers =
            VulkanContext::create_framebuffers(self.render_pass.clone(), &self.images);

        // Recreate Command Buffers
        let (_, new_command_buffers) = VulkanContext::create_command_pool_and_buffers(
            self.device.clone(),
            self.queue.id_within_family(),
            &self.framebuffers,
            self.render_pass.clone(),
        );

        self.command_buffers = new_command_buffers;

        log_info(state, "Swapchain recreated successfully");
    }

    // Function to Create Vertex Buffer Description
    pub fn create_vertex_buffer_description() -> VertexBufferDescription {
        // Define members for Vertex Buffer
        let mut members = HashMap::new();

        // Add vetex attributes
        members.insert(
            "position".to_string(),
            VertexMemberInfo {
            offset: 0,
            format: Format::R32G32B32_SFLOAT, // vec3 (xyz)
            num_elements: 1,
            },
        );
        members.insert(
            "color".to_string(),
            VertexMemberInfo {
                offset: 12,
                format: Format::R32G32B32A32_SFLOAT, // vec4 (rgba)
                num_elements: 1,
            },
        );

        // Define Vertex Buffer Description
        let vertex_buffer_description = VertexBufferDescription { 
            members,
            stride: 28, // Total size of one vertex (3 floats + 4 floats = 7 * 4 bytes)
            input_rate: VertexInputRate::Vertex,
        };

        vertex_buffer_description
    }

    // Function to Load Shader
    fn load_shader(device: Arc<Device>, shader_path: &str) -> Arc<ShaderModule> {
        let shader_code = std::fs::read(shader_path).expect("Failed to read shader file");

        // Convert Vec<u8> to Vec<u32>
        let shader_code_u32 = bytemuck::cast_slice::<u8, u32>(&shader_code);

        // Create ShaderModule
        ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(shader_code_32))
            .expect("Failed to create shader module")
    }

    // Function to Create Pipeline
    pub fn create_pipeline(&self) -> Arc<GraphicsPipeline> {
        // Load shaders
        let vertex_shaders = Self::load_shader(self.device.clone(), "shaders/vert.spv");
        let fragment_shaders = Self::load_shader(self.device.clone(), "shaders/frag.spv");

        // Call Render Pass
        let render_pass = Self::build_render_pass(self.device.clone());

        // Define pipeline layout
        let pipeline_layout = PipelineLayout::new(self.device.clone(), Default::default())
            .expect("Failed to create pipeline layout");

        // Define the viewport
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [800.0, 600.0],
            depth_range: 0.0..=1.0,
        };

        // Call Vertex Buffer Description
        let vertex_buffer_description = create_vertex_buffer_description();

        // Define PipelineCreateFlags
        let pipeline_create_flags =
            PipelineCreateFlags::DISABLE_OPTIMIZATION.union(PipelineCreateFlags::ALLOW_DERIVATIVES);

        // Define TessellationState
        let tessellation_state = TessellationState {
            patch_control_points: 4,
            domain_origin: TessellationDomainOrigin::UpperLeft,
            ..Default::default()
        };

        // Define Viewport State
        let viewport_state = ViewportState::default();

        // Define Dynamic State
        let dynamic_state = DynamicState:: 

        // Create the pipeline
        let pipeline_create_info = GraphicsPipelineCreateInfo {
            flags: pipeline_create_flags,
            //vertex_input_state: Some(VertexInputState::new()),
            vertex_input_state: Some(vertex_buffer_description.into()),
            input_assembly_state: Some(InputAssemblyState::default()),
            tessellation_state: Some(tessellation_state),
            viewport_state: Some(viewport_state),
            rasterization_state: Default::default(),
            multisample_state: Default::default(),
            color_blend_state: Default::default(),
            dynamic_state: 
            depth_stencil_state: Default::default(),
            layout: Arc::new(pipeline_layout),
            subpass: Some(Subpass::from(render_pass, 0).unwrap()),
            stages: vec![
                vertex_shader.entry_point("main").unwrap().into(),
                fragment_shader.entry_point("main").unwrap().into(),
            ]
            .into(),
        };

        // Create graphics pipeline
        GraphicsPipeline::new(self.device.clone(), None, pipeline_create_info)
            .expect("Failed to create graphics pipeline")
    }

    // Resizes the swapchain when the window size changes
    pub fn resize(&mut self, _new_width: u32, _new_height: u32, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Resizing to: {}x{}");
        // Handle resize logic here
    }

    /// Clean up Vulkan resources
    pub fn cleanup(&self, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Cleaning up Vulkan resources...");
    }
}
