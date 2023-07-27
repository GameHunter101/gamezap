use ash::{
    extensions,
    vk::{self, Format, Image, ImageView, SwapchainKHR},
};
use ash_bootstrap::{Swapchain as SwapchainBuilder, SwapchainOptions};

pub struct Swapchain {
    swapchain: SwapchainBuilder,
    format: Format,
    images: Vec<Image>,
}

impl Swapchain {
    pub fn init(
        extent: vk::Extent2D,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        swapchain_loader: extensions::khr::Swapchain,
    ) -> Self {
        let swapchain_options = SwapchainOptions::new()
            .present_mode_preference(&[vk::PresentModeKHR::FIFO])
            .frames_in_flight(2)
            .to_owned();

        let swapchain = SwapchainBuilder::new(
            swapchain_options,
            surface,
            physical_device,
            device,
            swapchain_loader,
            extent,
        );
        let format = swapchain.format().format;
        let images = swapchain.images().to_vec();

        Swapchain {
            swapchain,
            format,
            images,
        }
    }
}
