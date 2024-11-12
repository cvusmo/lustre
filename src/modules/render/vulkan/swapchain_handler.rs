// src/modules/render/vulkan/swapchain_handler.rs
// github.com/cvusmo/gameengine

use std::sync::Arc;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::image::{view::ImageView, ImageUsage};
use vulkano::swapchain::{
    CompositeAlpha, FullScreenExclusive, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
};
use vulkano::sync::SharingMode;

pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface>,
    queue: Arc<Queue>,
) -> Result<(Arc<Swapchain>, Vec<Arc<ImageView>>), Box<dyn std::error::Error>> {
    // Enable the swapchain extension on the device.
    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    // Query the surface capabilities.
    let surface_capabilities = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())?;

    // Use the current window size or a fixed resolution.
    let image_extent = surface_capabilities.current_extent.unwrap_or([640, 480]);

    // Set the minimum image count for double-buffering.
    let min_image_count = match surface_capabilities.max_image_count {
        None => std::cmp::max(2, surface_capabilities.min_image_count),
        Some(limit) => std::cmp::min(
            std::cmp::max(2, surface_capabilities.min_image_count),
            limit,
        ),
    };

    // Preserve the current surface transform.
    let pre_transform = surface_capabilities.current_transform;

    // Use the first available format.
    let (image_format, _color_space) = device
        .physical_device()
        .surface_formats(&surface, Default::default())?[0];

    // Create the swapchain with the desired parameters.
    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count,
            image_format,
            image_extent,
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            image_sharing: SharingMode::Exclusive(vec![queue.family()]),
            pre_transform,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode: PresentMode::Fifo, // Use FIFO as the default vsync mode.
            full_screen_exclusive: FullScreenExclusive::Default,
            ..Default::default()
        },
    )?;

    // Wrap the swapchain images in `ImageView`.
    let image_views = images
        .into_iter()
        .map(|image| ImageView::new_default(image).unwrap())
        .collect();

    Ok((swapchain, image_views))
}
