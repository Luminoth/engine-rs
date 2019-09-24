use std::sync::Arc;

use failure::{bail, format_err};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::FormatDesc;
use vulkano::image::{Dimensions, StorageImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::memory::Content;
use vulkano::swapchain::{PresentMode, Surface, SurfaceTransform, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::*;

pub struct VulkanRenderer {
    instance: Arc<Instance>,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,

    swapchain: Arc<Swapchain<Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
}

impl VulkanRenderer {
    pub fn new(events_loop: &EventsLoop) -> Result<Self> {
        let extensions = vulkano_win::required_extensions();
        // TODO: what about application-required extensions?

        println!("Initializing vulkan renderer...
\tExtensions: {:?}", extensions);
        let instance = Instance::new(None, &extensions, None)?;

        // TODO: need to do application requirement filtering here
        // and should allow the application to select between
        // all devices that fit within those constraints
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

        println!("Creating surface...");
        let surface = WindowBuilder::new().build_vk_surface(events_loop, instance.clone())?;

        let graphics_queue_family = physical_device
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .ok_or_else(|| format_err!("No graphics queues available!"))?;

        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        println!("Creating device...");
        let (device, mut graphics_queues) = Device::new(
            physical_device,
            physical_device.supported_features(),
            &device_ext,
            [(graphics_queue_family, 0.5)].iter().cloned(),
        )?;
        let graphics_queue = graphics_queues.next().unwrap();

        let capabilities = surface.capabilities(physical_device)?;
        let alpha = capabilities
            .supported_composite_alpha
            .iter()
            .next()
            .unwrap();
        let format = capabilities.supported_formats[0].0;

        let initial_dimensions = if let Some(dimensions) = surface.window().get_inner_size() {
            // convert to physical pixels
            let dimensions: (u32, u32) = dimensions
                .to_physical(surface.window().get_hidpi_factor())
                .into();
            [dimensions.0, dimensions.1]
        } else {
            bail!("Window no longer exists!");
        };

        println!("Creating swapchain...");
        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            capabilities.min_image_count,
            format,
            initial_dimensions,
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

    //#region CPU Buffers

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

    pub fn create_cpu_buffer_iter<V, T>(&self, data: V) -> Result<Arc<CpuAccessibleBuffer<[T]>>>
    where
        V: AsRef<Vec<T>>,
        T: Content + Clone + 'static,
    {
        Ok(CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            data.as_ref().iter().cloned(),
        )?)
    }

    pub fn create_vertex_buffer<V>(&self, vertices: V) -> Result<Arc<CpuAccessibleBuffer<[Vertex]>>>
    where
        V: AsRef<Vec<Vertex>>,
    {
        self.create_cpu_buffer_iter(vertices)
    }

    //#endregion

    //#region Images

    pub fn create_image_2d<F>(
        &self,
        width: u32,
        height: u32,
        format: F,
    ) -> Result<Arc<StorageImage<F>>>
    where
        F: FormatDesc,
    {
        Ok(StorageImage::new(
            self.device.clone(),
            Dimensions::Dim2d { width, height },
            format,
            Some(self.graphics_queue.family()),
        )?)
    }

    //#endregion

    //#region Shaders

    // TODO: probably have to customize this so we have a trait to genericize against

    pub fn load_simple_shader(&self) -> Result<shaders::simple::vs::Shader> {
        Ok(shaders::simple::vs::Shader::load(self.device.clone())?)
    }

    //#endregion

    //#region Command Buffers

    pub fn create_command_buffer(
        &self,
    ) -> Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::new(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }

    //#endregion

    /*pub fn create_simple_render_pass(&self) {
        Ok(Arc::new(vulkano::single_pass_renderpass!(
            self.device.clone(),
            attachments,
            pass
        )?))
    }*/
}

impl Renderer for VulkanRenderer {}
