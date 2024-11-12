// src/modules/render/vulkan/swapchain.rs
// github.com/cvusmo/gameengine

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::swapchain::Surface;
use vulkano::VulkanError;
use winit::window::Window;

pub fn create_vulkan_surface(
    instance: Arc<Instance>,
    window: &Window,
) -> Result<Arc<Surface>, VulkanError> {
    // Ensure the window handle is for Wayland
    let handle = window.raw_window_handle();
    let display_handle = window.raw_display_handle();

    if let (
        raw_window_handle::RawWindowHandle::Wayland(window),
        raw_window_handle::RawDisplayHandle::Wayland(display),
    ) = (handle, display_handle)
    {
        // SAFETY: Ensure display and surface handles are valid and outlive the surface
        unsafe {
            Surface::from_wayland(
                instance.clone(),
                display.display, // Wayland display handle
                window.surface,  // Wayland surface handle
                Some(Arc::new(window.clone())),
            )
        }
    } else {
        Err(VulkanError::InvalidWindowHandle) // Or a custom error if you prefer
    }
}
