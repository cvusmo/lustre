// src/modules/render/vulkan/swapchain_handler.rs
// github.com/cvusmo/gameengine

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::image::ImageUsage;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::sync::SharingMode;
use winit::window::Window;

pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
) -> (
    Arc<Swapchain<Window>>,
    Vec<Arc<vulkano::image::ImageView<SwapchainImage<Window>>>>,
) {
    let caps = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .expect("Failed to get surface capabilities");

    let dimensions = surface.window().inner_size();

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_format: Some(caps.supported_formats[0].0),
            image_extent: [dimensions.width, dimensions.height],
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha: caps.current_transform,
            ..Default::default()
        },
    )
    .expect("Failed to create swapchain");

    (swapchain, images)
}
