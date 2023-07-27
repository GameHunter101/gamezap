use std::{ffi, rc::Rc};

use ash::{
    extensions,
    vk::{self, Handle},
    Entry, Instance,
};
use ash_bootstrap::{
    DeviceBuilder, DeviceMetadata, InstanceBuilder, InstanceMetadata, QueueFamilyCriteria,
};
use sdl2::{event::Event, keyboard::Keycode, video::Window};

use crate::swapchain::Swapchain;

pub struct VulkanEngine {
    window_name: &'static str,
    initialized: bool,
    frame_number: i32,

    window: Rc<Window>,
    sdl_context: sdl2::Sdl,

    vulkan_items: VulkanItems,
}

impl VulkanEngine {
    pub fn init(
        window_name: &'static str,
        width: u32,
        height: u32,
    ) -> Result<VulkanEngine, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init().unwrap();

        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(window_name, width, height)
            .vulkan()
            .build()
            .unwrap();

        let window_ref = Rc::new(window);
        let mut vulkan_items = VulkanItems::init(width, height, window_name, window_ref.clone())?;

        vulkan_items.create_main_objects()?;

        Ok(VulkanEngine {
            window_name,
            initialized: true,
            frame_number: 0,
            window: window_ref.clone(),
            sdl_context,
            vulkan_items,
        })
    }

    pub fn cleanup(&self) {
        if self.initialized {}
    }

    pub fn draw(&self) {}

    pub fn run(&self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {}
                }
            }
            self.draw();
            ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}

pub struct VulkanItems {
    entry: Entry,
    window_extent: vk::Extent2D,
    window_name: &'static str,
    window: Rc<Window>,

    instance: Option<ash::Instance>,
    instance_metadata: Option<InstanceMetadata>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    device: Option<ash::Device>,
    physical_device: Option<vk::PhysicalDevice>,
    surface: Option<vk::SurfaceKHR>,

    swapchain: Option<Swapchain>,
}

impl VulkanItems {
    pub fn init(
        width: u32,
        height: u32,
        window_name: &'static str,
        window: Rc<Window>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let window_extent = vk::Extent2D { width, height };
        let entry = unsafe { Entry::load()? };

        Ok(VulkanItems {
            entry,
            window_extent,
            window_name,
            window,

            instance: None,
            instance_metadata: None,
            debug_messenger: None,
            device: None,
            physical_device: None,
            surface: None,

            swapchain: None,
        })
    }

    pub fn create_main_objects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (instance, instance_metadata, debug_messenger) = self.init_instance()?;
        self.instance = Some(instance);
        self.instance_metadata = Some(instance_metadata);
        self.debug_messenger = Some(debug_messenger);
        let surface = self.init_surface();
        self.surface = Some(surface);
        let (device, device_metadata) = self.init_device().unwrap();
        let physical_device = device_metadata.physical_device();

        let swapchain = Swapchain::init(
            self.window_extent,
            surface,
            physical_device,
            &device,
            extensions::khr::Swapchain::new(&instance, &device),
        );

        self.device = Some(device);
        self.physical_device = Some(physical_device);
        

        Ok(())
    }

    pub fn init_instance(
        &self,
    ) -> Result<(Instance, InstanceMetadata, vk::DebugUtilsMessengerEXT), vk::Result> {
        let (instance, (debug_loader, debug_messenger), instance_metadata) = unsafe {
            InstanceBuilder::new()
                .validation_layers(ash_bootstrap::ValidationLayers::Request)
                .require_api_version(1, 1)
                .request_debug_messenger(ash_bootstrap::DebugMessenger::Default)
                .build(&self.entry)
                .unwrap()
        };
        Ok((instance, instance_metadata, debug_messenger.unwrap()))
    }

    fn init_surface(&self) -> vk::SurfaceKHR {
        let surface_id = (*self.window)
            .vulkan_create_surface(self.instance.clone().unwrap().handle().as_raw() as usize)
            .unwrap();
        let surface = vk::SurfaceKHR::from_raw(surface_id);
        surface
    }

    fn init_device(&self) -> Result<(ash::Device, DeviceMetadata), vk::Result> {
        let graphics_present = QueueFamilyCriteria::graphics_present();
        let device_builder = DeviceBuilder::new()
            .require_extension(extensions::khr::Swapchain::name().as_ptr())
            .queue_family(graphics_present)
            .for_surface(self.surface.unwrap());
        let surface_loader =
            extensions::khr::Surface::new(&self.entry, &self.instance.as_ref().unwrap());

        let (device, device_metadata) = unsafe {
            device_builder
                .build(
                    &self.instance.as_ref().unwrap(),
                    &surface_loader,
                    &self.instance_metadata.as_ref().unwrap(),
                )
                .unwrap()
        };
        Ok((device, device_metadata))
    }
}
