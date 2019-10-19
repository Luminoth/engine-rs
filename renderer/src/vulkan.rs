use std::sync::Arc;

use anyhow::{anyhow, bail};
use derivative::Derivative;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::format::FormatDesc;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass};
use vulkano::image::{Dimensions, StorageImage, SwapchainImage};
use vulkano::instance::debug::DebugCallback;
use vulkano::instance::{Instance, InstanceExtensions, LayerProperties, PhysicalDevice};
use vulkano::memory::Content;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainAcquireFuture,
    SwapchainCreationError,
};
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::{EventsLoop, Window, WindowBuilder};

use crate::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct VulkanRendererState {
    instance: Arc<Instance>,

    #[derivative(Debug = "ignore")]
    debug_callback: Option<DebugCallback>,

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
    pub fn new(events_loop: &EventsLoop) -> anyhow::Result<Self> {
        // TODO: pass in the values rather than pulling from cargo
        let app_info = vulkano::app_info_from_cargo_toml!();

        let supported_instance_extensions = InstanceExtensions::supported_by_core()?;

        let mut extensions = vulkano_win::required_extensions();
        // TODO: what about application-required extensions?

        if cfg!(feature = "validation") {
            println!("Enabling debug reporting...");
            extensions.ext_debug_report = true;
        }

        let supported_layers: Vec<LayerProperties> = vulkano::instance::layers_list()?.collect();

        // TODO: layers would be better dealt with using a combination
        // of features and VK_INSTANCE_LAYERS to turn specific ones on or off

        let mut layers = Vec::new();
        if cfg!(feature = "validation") {
            println!("Enabling validation...");
            layers.push("VK_LAYER_LUNARG_core_validation");
            layers.push("VK_LAYER_LUNARG_standard_validation");
        }

        if cfg!(feature = "vktrace") {
            println!("Enabling vktrace...");
            layers.push("VK_LAYER_LUNARG_vktrace");
        }

        if cfg!(feature = "renderdoc") {
            println!("Enabling renderdoc capture...");
            layers.push("VK_LAYER_RENDERDOC_Capture");
        }

        println!(
            "Initializing Vulkan renderer...
\tApp Info: {:?}
\tSupported Extensions: {:?}
\tRequested Extensions: {:?}
\tSupported Layers: {:?}
\tRequested Layers: {:?}",
            app_info,
            supported_instance_extensions,
            extensions,
            supported_layers
                .iter()
                .map(|x| x.name())
                .collect::<Vec<_>>(),
            layers,
        );
        let instance = Instance::new(Some(&app_info), &extensions, layers)?;

        let debug_callback = if cfg!(feature = "validation") {
            Some(DebugCallback::errors_and_warnings(&instance, |msg| {
                eprintln!("Debug callback: {:?}", msg.description);
            })?)
        } else {
            None
        };

        println!("Creating surface...");
        let surface = WindowBuilder::new().build_vk_surface(events_loop, instance.clone())?;

        // TODO: need to do application requirement filtering here
        // and should allow the application to select between
        // all devices that fit within those constraints
        let physical_device = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or_else(|| anyhow!("No devices available!"))?;

        let supported_device_extensions = DeviceExtensions::supported_by_device(physical_device);

        println!(
            "Got physical device:
\tName: {}
\tType: {:?}
\tAPI Version: {}
\tFeatures: {:?}
\tSupported Extensions: {:?}",
            physical_device.name(),
            physical_device.ty(),
            physical_device.api_version(),
            physical_device.supported_features(),
            supported_device_extensions,
        );

        let graphics_queue_family = physical_device
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .ok_or_else(|| anyhow!("No graphics queues available!"))?;

        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        println!("Creating logical device...");
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

        // TODO: use Fifo if vsync
        let present_mode = if capabilities.present_modes.supports(PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else {
            PresentMode::Fifo
        };

        // TODO: comb over https://vulkan-tutorial.com/en/Drawing_a_triangle/Presentation/Swap_chain

        println!("Creating swapchain...");
        let (swapchain, swapchain_images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            capabilities.min_image_count + 1,
            format,
            crate::get_window_dimensions(surface.window())?,
            1,
            capabilities.supported_usage_flags,
            &graphics_queue,
            SurfaceTransform::Identity,
            alpha,
            present_mode,
            true,
            None,
        )?;

        Ok(Self {
            instance,
            debug_callback,
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

    pub(crate) fn recreate_swapchain(&mut self) -> anyhow::Result<bool> {
        println!("Recreating swapchain...");
        let (new_swapchain, new_images) = match self
            .swapchain
            .recreate_with_dimension(crate::get_window_dimensions(self.surface.window())?)
        {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::UnsupportedDimensions) => return Ok(false),
            Err(err) => bail!(err),
        };

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images;

        Ok(true)
    }

    //#region CPU Buffers

    pub fn create_cpu_buffer<T>(&self, data: T) -> anyhow::Result<Arc<CpuAccessibleBuffer<T>>>
    where
        T: Content + 'static,
    {
        Ok(CpuAccessibleBuffer::from_data(
            self.device.clone(),
            BufferUsage::all(),
            data,
        )?)
    }

    pub fn create_cpu_buffer_iter<V, T>(
        &self,
        data: V,
    ) -> anyhow::Result<Arc<CpuAccessibleBuffer<[T]>>>
    where
        V: Into<Vec<T>>,
        T: Content + Clone + 'static,
    {
        Ok(CpuAccessibleBuffer::from_iter(
            self.device.clone(),
            BufferUsage::all(),
            data.into().iter().cloned(),
        )?)
    }

    //#endregion

    //#region Images

    pub fn create_image_2d<F>(
        &self,
        width: u32,
        height: u32,
        format: F,
    ) -> anyhow::Result<Arc<StorageImage<F>>>
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
    ) -> anyhow::Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::new(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }

    pub(crate) fn create_primary_one_time_submit_command_buffer(
        &self,
    ) -> anyhow::Result<AutoCommandBufferBuilder<StandardCommandPoolBuilder>> {
        Ok(AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.graphics_queue.family(),
        )?)
    }

    //#endregion

    //#region Render Pass

    pub(crate) fn create_simple_render_pass(&self) -> anyhow::Result<RenderPass> {
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
    ) -> anyhow::Result<Vec<FrameBuffer>> {
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
            _ => bail!("Render pass type {} not supported", render_pass),
        })
    }

    //#endregion

    //#region Pipeline

    pub(crate) fn create_simple_render_pipeline(
        &self,
        render_pass: &RenderPass,
        vs: shaders::simple::vs::Shader,
        fs: shaders::simple::fs::Shader,
    ) -> anyhow::Result<RenderPipeline> {
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
            _ => bail!("Render pass type {} not supported", render_pass),
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

    fn acquire_swapchain(&mut self) -> anyhow::Result<Option<SwapchainAcquireFuture<Window>>> {
        let (swapchain_image, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(result) => result,
                Err(AcquireError::OutOfDate) => {
                    return Ok(None);
                }
                Err(e) => bail!(e),
            };

        self.current_swapchain_image = swapchain_image;

        Ok(Some(acquire_future))
    }

    pub(crate) fn draw_data<F>(
        &mut self,
        render_pipeline: &RenderPipeline,
        clear_values: [f32; 4],
        draw_data: &VertexBuffer,
        frame_buffers: F,
    ) -> anyhow::Result<bool>
    where
        F: AsRef<Vec<FrameBuffer>>,
    {
        let acquire_future = self.acquire_swapchain()?;
        if acquire_future.is_none() {
            return Ok(false);
        }
        let acquire_future = acquire_future.unwrap();

        let frame_buffers = frame_buffers.as_ref();
        let frame_buffer = &frame_buffers[self.current_swapchain_image];

        let clear_values = vec![clear_values.into()];

        let command_buffer = self
            .create_primary_one_time_submit_command_buffer()?
            .begin_render_pass(
                match frame_buffer {
                    FrameBuffer::Vulkan(f) => f,
                    FrameBuffer::None => bail!("Invalid framebuffer type {}", frame_buffer),
                }
                .clone(),
                false,
                clear_values,
            )?
            .draw(
                match render_pipeline {
                    RenderPipeline::Vulkan(p) => p,
                    RenderPipeline::None => {
                        bail!("Invalid render pipeline type {}", render_pipeline)
                    }
                }
                .clone(),
                &self.dynamic_state,
                vec![match draw_data {
                    VertexBuffer::Vulkan(v) => v,
                    VertexBuffer::None => bail!("Invalid vertex buffer type {}", draw_data),
                }
                .clone()],
                (),
                (),
            )?
            .end_render_pass()?
            .build()?;

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
