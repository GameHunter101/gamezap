mod utils;
mod vulkan_engine;
mod swapchain;

use vulkan_engine::VulkanEngine;

fn main() {
    let engine = VulkanEngine::init("GameZap", 800, 600).unwrap();

    engine.run();

    engine.cleanup();
}
