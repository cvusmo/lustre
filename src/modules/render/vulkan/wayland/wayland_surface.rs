// src/modules/render/vulkan/wayland/wayland_surface.rs
// github.com/cvusmo/gameengine

use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano::VulkanError;
use winit::platform::wayland::WindowExtWayland;
use winit::window::Window;

pub fn create_vulkan_surface_wayland(
    instance: Arc<Instance>,
    window: &Window,
) -> Result<Arc<Surface>, VulkanError> {
    // Use Wayland-specific methods provided by the `WindowExtWayland` trait
    if let (Some(wayland_display), Some(wayland_surface)) =
        (window.wayland_display(), window.wayland_surface())
    {
        // SAFETY: Ensure that display and surface handles are valid and outlive the surface
        unsafe {
            Surface::from_wayland(
                instance.clone(),
                wayland_display, // Wayland display handle
                wayland_surface, // Wayland surface handle
                None,
            )
        }
    } else {
        Err(VulkanError::InvalidWindowHandle)
    }
}
