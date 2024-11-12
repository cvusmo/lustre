// src/modules/render/vulkan/swapchain.rs
// github.com/cvusmo/gameengine

use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use winit::window::Window;

pub fn create_vulkan_surface(instance: Arc<Instance>, window: &Window) -> Arc<Surface<Window>> {
    Surface::from_window(instance, window.clone()).expect("Failed to create Vulkan surface")
}

