use std::sync::Arc;

use derivative::Derivative;
use failure::{bail, format_err, Error};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::FormatDesc;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass};
use vulkano::image::{Dimensions, StorageImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::memory::Content;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
};
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::*;

type VertexBuffer = Arc<CpuAccessibleBuffer<[Vertex]>>;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct VulkanRendererState {
    instance: Arc<Instance>,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,

    #[derivative(Debug = "ignore")]
    swapchain: Arc<Swapchain<Window>>,

    #[derivative(Debug = "ignore")]
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,

    current_swapchain_image: usize,

    dynamic_state: DynamicState,

    #[derivative(Debug = "ignore")]
    frame_future: Option<Box<dyn GpuFuture>>,
}

impl VulkanRendererState {
    pub fn new(events_loop: &EventsLoop) -> Result<Self> {
        let extensions = vulkano_win::required_extensions();
        // TODO: what about application-required extensions?

        println!(
            "Initializing Vulkan renderer...
\tExtensions: {:?}",
            extensions
        );
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
            current_swapchain_image: 0,
            dynamic_state: DynamicState::none(),
            frame_future: None,
        })
    }

    pub(crate) fn get_device(&self) -> &Arc<Device> {
        &self.device
    }

    pub(crate) fn get_window(&self) -> &Window {
        self.surface.window()
    }

    pub(crate) fn init_viewport(&mut self) {
        let dimensions = self.swapchain_images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        self.dynamic_state.viewports = Some(vec![viewport]);
    }

    pub(crate) fn get_current_swapchain_image(&self) -> usize {
        self.current_swapchain_image
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

    pub fn create_vertex_buffer<V>(&self, vertices: V) -> Result<VertexBuffer>
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

    //#region Command Buffers

    pub fn create_command_buffer(
        &self,
    ) -> Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::new(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }

    pub(crate) fn create_primary_one_time_submit_command_buffer(
        &self,
    ) -> Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }

    //#endregion

    //#region Render Pass

    pub(crate) fn create_simple_render_pass(&self) -> Result<RenderPass> {
        Ok(RenderPass::Vulkan(Arc::new(
            vulkano::single_pass_renderpass!(
               self.device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: self.swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )?,
        )))
    }

    //#endregion

    //#region Frame Buffers

    pub(crate) fn create_frame_buffers(
        &mut self,
        render_pass: &RenderPass,
    ) -> Result<Vec<FrameBuffer>> {
        Ok(match render_pass {
            RenderPass::Vulkan(rp) => {
                let mut frame_buffers = Vec::new();
                for image in &self.swapchain_images {
                    let frame_buffer = FrameBuffer::Vulkan(Arc::new(
                        Framebuffer::start(rp.clone()).add(image.clone())?.build()?,
                    )
                        as Arc<dyn FramebufferAbstract + Send + Sync>);
                    frame_buffers.push(frame_buffer);
                }

                frame_buffers
            }
            _ => bail!("Render pass type {} not supported!", render_pass),
        })
    }

    //#endregion

    //#region Pipeline

    pub(crate) fn create_simple_render_pipeline(
        &self,
        render_pass: &RenderPass,
        vs: shaders::simple::vs::Shader,
        fs: shaders::simple::fs::Shader,
    ) -> Result<RenderPipeline> {
        Ok(match render_pass {
            RenderPass::Vulkan(rp) => RenderPipeline::Vulkan(Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .fragment_shader(fs.main_entry_point(), ())
                    .render_pass(Subpass::from(rp.clone(), 0).unwrap())
                    .build(self.device.clone())?,
            )),
            _ => bail!("Render pass type {} not supported!", render_pass),
        })
    }

    //#endregion

    pub(crate) fn begin_frame(&mut self) {
        match &mut self.frame_future {
            Some(ref mut frame_future) => frame_future.cleanup_finished(),
            None => {
                self.frame_future =
                    Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<dyn GpuFuture>)
            }
        }
    }

    pub(crate) fn acquire_swapchain(&mut self) -> Result<Option<SwapchainAcquireFuture<Window>>> {
        let (swapchain_image, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    return Ok(None);
                }
                Err(e) => return Err(Error::from(e)),
            };

        self.current_swapchain_image = swapchain_image;
        Ok(Some(acquire_future))
    }

    pub(crate) fn submit(
        &mut self,
        acquire_future: SwapchainAcquireFuture<Window>,
        command_buffer: AutoCommandBuffer,
    ) -> Result<bool> {
        let future = self
            .frame_future
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.graphics_queue.clone(),
                self.swapchain.clone(),
                self.current_swapchain_image,
            )
            .then_signal_fence_and_flush();

        let mut recreate_swapchain = false;
        match future {
            Ok(future) => {
                self.frame_future = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                self.frame_future =
                    Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.frame_future =
                    Some(Box::new(vulkano::sync::now(self.device.clone())) as Box<_>);
            }
        }

        Ok(!recreate_swapchain)
    }
}
