use std::sync::Arc;

use failure::format_err;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::memory::Content;
use vulkano::swapchain::{PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::Result;

pub struct VkRenderer {
    instance: Arc<Instance>,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,

    swapchain: Arc<Swapchain<Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
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
            "Got physical device:
\tName: {}
\tType: {:?}
\tAPI Version: {}
\tFeatures: {:?}",
            physical_device.name(),
            physical_device.ty(),
            physical_device.api_version(),
            physical_device.supported_features(),
        );

        let graphics_queue_family = physical_device
            .queue_families()
            .find(|&q| q.supports_graphics())
            .ok_or_else(|| format_err!("No graphics queues available!"))?;

        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let (device, mut graphics_queues) = Device::new(
            physical_device,
            &Features::none(),
            &device_ext,
            [(graphics_queue_family, 0.5)].iter().cloned(),
        )?;
        let graphics_queue = graphics_queues.next().unwrap();

        println!("Creating surface...");
        let surface = WindowBuilder::new().build_vk_surface(events_loop, instance.clone())?;

        let capabilities = surface.capabilities(physical_device)?;
        let dimensions = capabilities.current_extent.unwrap_or([1280, 1024]);
        let alpha = capabilities
            .supported_composite_alpha
            .iter()
            .next()
            .unwrap();
        let format = capabilities.supported_formats[0].0;

        println!("Creating swapchain...");
        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            capabilities.min_image_count,
            format,
            dimensions,
            1,
            capabilities.supported_usage_flags,
            &graphics_queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )?;

        Ok(Self {
            instance,
            device,
            graphics_queue,
            surface,
            swapchain,
            swapchain_images,
        })
    }

    pub fn get_instance(&self) -> &Arc<Instance> {
        &self.instance
    }

    pub fn get_window(&self) -> &Window {
        self.surface.window()
    }

    pub fn create_cpu_buffer<T>(&self, data: T) -> Result<Arc<CpuAccessibleBuffer<T>>>
    where
        T: Content + 'static,
    {
        Ok(CpuAccessibleBuffer::from_data(
            self.device.clone(),
            BufferUsage::all(),
            data,
        )?)
    }

    pub fn create_command_buffer(
        &self,
    ) -> Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::new(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }
}
