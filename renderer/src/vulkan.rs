use std::sync::Arc;

use failure::format_err;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::Result;

pub struct VkRenderer {
    instance: Arc<Instance>,
    surface: Arc<Surface<Window>>,
}

impl VkRenderer {
    pub fn new(events_loop: &EventsLoop) -> Result<Self> {
        println!("Initializing vulkan...");

        let extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, &extensions, None)?;

        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or_else(|| format_err!("No devices available!"))?;
        println!(
            "Got device:\n\tName: {}\n\tType: {:?}\n\tAPI Version: {}\n\tFeatures: {:?}",
            physical_device.name(),
            physical_device.ty(),
            physical_device.api_version(),
            physical_device.supported_features(),
        );

        let queue_family = physical_device
            .queue_families()
            .find(|&q| q.supports_graphics())
            .ok_or_else(|| format_err!("No graphics queues available!"))?;

        let (_device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )?;
        let _queue = queues.next().unwrap();

        println!("Creating window...");
        let surface = WindowBuilder::new().build_vk_surface(events_loop, instance.clone())?;

        Ok(Self { instance, surface })
    }

    pub fn get_instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    pub fn get_surface(&self) -> &Arc<Surface<Window>> {
        &self.surface
    }
}
