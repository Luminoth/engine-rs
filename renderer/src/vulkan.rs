use std::sync::Arc;

use failure::{bail, format_err, Error};
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::{ClearValue, FormatDesc};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::{Dimensions, StorageImage, SwapchainImage};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::memory::Content;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
};
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::*;

type VertexBuffer = Arc<CpuAccessibleBuffer<[Vertex]>>;

pub struct VulkanRenderPass {
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
}

impl VulkanRenderPass {
    pub fn new(render_pass: Arc<dyn RenderPassAbstract + Send + Sync>) -> Self {
        Self { render_pass }
    }
}

impl crate::RenderPass for VulkanRenderPass {}

pub struct VulkanPipeline {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

impl VulkanPipeline {
    fn new(pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>) -> Self {
        Self { pipeline }
    }
}

impl crate::Pipeline for VulkanPipeline {}

pub struct VulkanRenderer {
    instance: Arc<Instance>,

    device: Arc<Device>,

    graphics_queue: Arc<Queue>,

    surface: Arc<Surface<Window>>,

    swapchain: Arc<Swapchain<Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    current_swapchain_image: usize,

    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,

    dynamic_state: DynamicState,

    frame_end: Box<dyn GpuFuture>,
}

impl VulkanRenderer {
    pub fn new(events_loop: &EventsLoop) -> Result<Self> {
        let extensions = vulkano_win::required_extensions();
        // TODO: what about application-required extensions?

        println!(
            "Initializing vulkan renderer...
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

        let frame_end = Box::new(vulkano::sync::now(device.clone()));

        Ok(Self {
            instance,
            device,
            graphics_queue,
            surface,
            swapchain,
            swapchain_images,
            current_swapchain_image: 0,
            framebuffers: Vec::new(),
            dynamic_state: DynamicState::none(),
            frame_end,
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

    //#region Shaders

    // TODO: probably have to customize this so we have a trait to genericize against

    pub fn load_simple_shader(
        &self,
    ) -> Result<(shaders::simple::vs::Shader, shaders::simple::fs::Shader)> {
        let vs = shaders::simple::vs::Shader::load(self.device.clone())?;
        let fs = shaders::simple::fs::Shader::load(self.device.clone())?;
        Ok((vs, fs))
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

    //#region Frame Buffers

    pub fn create_frame_buffers(&mut self, render_pass: Arc<dyn RenderPassAbstract + Send + Sync>) {
        let dimensions = self.swapchain_images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        self.dynamic_state.viewports = Some(vec![viewport]);

        self.framebuffers = self
            .swapchain_images
            .iter()
            .map(|image| {
                Arc::new(
                    Framebuffer::start(render_pass.clone())
                        .add(image.clone())
                        .unwrap()
                        .build()
                        .unwrap(),
                ) as Arc<dyn FramebufferAbstract + Send + Sync>
            })
            .collect::<Vec<_>>();
    }

    //#endregion

    pub fn create_simple_render_pass(&self) -> Result<VulkanRenderPass> {
        println!("Creating simple render pass...");

        Ok(VulkanRenderPass::new(Arc::new(
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

    pub fn create_simple_pipeline(
        &self,
        render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
        vs: shaders::simple::vs::Shader,
        fs: shaders::simple::fs::Shader,
    ) -> Result<VulkanPipeline> {
        println!("Creating simple pipeline...");

        Ok(VulkanPipeline::new(Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(Subpass::from(render_pass, 0).unwrap())
                .build(self.device.clone())?,
        )))
    }

    fn acquire_swapchain(&mut self) -> Result<Option<SwapchainAcquireFuture<Window>>> {
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

    pub fn draw_data<B>(
        &mut self,
        pipeline: &VulkanPipeline,
        clear_values: Vec<f32>,
        draw_data: Vec<Arc<dyn BufferAccess + Send + Sync>>,
    ) -> Result<bool> {
        let acquire_future = self.acquire_swapchain()?;
        if acquire_future.is_none() {
            return Ok(false);
        }
        let acquire_future = acquire_future.unwrap();

        let mut cv = Vec::new();
        for v in clear_values {
            cv.push(ClearValue::from(v));
        }

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.graphics_queue.family(),
        )?
        .begin_render_pass(
            self.framebuffers[self.current_swapchain_image].clone(),
            false,
            cv,
        )?
        .draw(
            pipeline.pipeline.clone(),
            &self.dynamic_state,
            draw_data,
            (),
            (),
        )?
        .end_render_pass()?
        .build()?;

        /*let future = self
            .frame_end
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.graphics_queue.clone(),
                self.swapchain.clone(),
                self.current_swapchain_image,
            )
            .then_signal_fence_and_flush();*/

        let mut recreate_swapchain = false;
        /*match future {
            Ok(future) => {
                self.frame_end = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                self.frame_end = Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                self.frame_end = Box::new(vulkano::sync::now(self.device.clone())) as Box<_>;
            }
        }*/

        Ok(!recreate_swapchain)
    }
}

impl Renderer for VulkanRenderer {
    fn begin_frame(&mut self) {
        self.frame_end.cleanup_finished();
    }
}
