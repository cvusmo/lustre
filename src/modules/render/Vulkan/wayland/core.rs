// src/modules/engine/render/Vulkan/wayland/core.rs

use std::sync::{Arc, Mutex};
use std::time::Duration;
use vulkano::command_buffer::pool::{CommandPool, CommandPoolCreateInfo};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateFlags, QueueCreateInfo,
    QueueFlags,
};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo};
use vulkano::image::ImageUsage;
use vulkano::instance::Instance;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
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
    fn create_render_pass(device: Arc<Device>) -> Arc<RenderPass> {
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

    /// Render function placeholder for drawing frames
    pub fn render(&mut self, state: &Arc<Mutex<AppState>>) {
        log_info(state, "Rendering a frame...");

        let (image_index, suboptimal, acquire_future) = match swapchain::acquire_next_image(
            self.swapchain.clone(),
            Some(Duration::from_secs(1)),
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

        // Getter for command_buffer for acquired image index
        let command_buffer = &self.command_buffers[image_index as usize];

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
