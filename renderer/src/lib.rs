pub mod shaders;
mod vulkan;

use std::fmt;
use std::sync::Arc;

use derivative::Derivative;
use failure::{bail, Error};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::pipeline::GraphicsPipelineAbstract;

pub use vulkan::VulkanRendererState;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Derivative)]
#[derivative(Default)]
pub enum RenderPass {
    Vulkan(Arc<dyn RenderPassAbstract + Send + Sync>),

    #[derivative(Default)]
    None,
}

impl fmt::Display for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderPass::Vulkan(_) => write!(f, "Vulkan"),
            RenderPass::None => write!(f, "None"),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub enum FrameBuffer {
    Vulkan(Arc<dyn FramebufferAbstract + Send + Sync>),

    #[derivative(Default)]
    None,
}

impl fmt::Display for FrameBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrameBuffer::Vulkan(_) => write!(f, "Vulkan"),
            FrameBuffer::None => write!(f, "None"),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub enum RenderPipeline {
    Vulkan(Arc<dyn GraphicsPipelineAbstract + Send + Sync>),

    #[derivative(Default)]
    None,
}

impl fmt::Display for RenderPipeline {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderPipeline::Vulkan(_) => write!(f, "Vulkan"),
            RenderPipeline::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub enum Renderer {
    Vulkan(VulkanRendererState),

    #[derivative(Default)]
    None,
}

impl Renderer {
    //#region Window Utils

    pub fn set_window_title<S>(&self, window_title: S)
    where
        S: Into<String>,
    {
        match self {
            Renderer::Vulkan(r) => r.get_window().set_title(&window_title.into()),
            Renderer::None => (),
        }
    }

    //#endregion

    fn init_viewport(&mut self) {
        match self {
            Renderer::Vulkan(r) => r.init_viewport(),
            Renderer::None => (),
        }
    }

    //#region Shaders

    // TODO: probably have to customize this so we have a trait to genericize against

    pub fn load_simple_shader(
        &self,
    ) -> Result<(shaders::simple::vs::Shader, shaders::simple::fs::Shader)> {
        println!("Loading simple shaders...");

        Ok(match self {
            Renderer::Vulkan(r) => (
                shaders::simple::vs::Shader::load(r.get_device().clone())?,
                shaders::simple::fs::Shader::load(r.get_device().clone())?,
            ),
            Renderer::None => bail!("Shaders not supported!"),
        })
    }

    //#endregion

    //#region Render Pass

    pub fn create_simple_render_pass(&self) -> Result<RenderPass> {
        println!("Creating simple render pass...");

        Ok(match self {
            Renderer::Vulkan(r) => r.create_simple_render_pass()?,
            Renderer::None => RenderPass::None,
        })
    }

    //#endregion

    //#region Frame Buffers

    pub fn create_frame_buffers(&mut self, render_pass: &RenderPass) -> Result<Vec<FrameBuffer>> {
        println!("Creating frame buffers...");

        self.init_viewport();

        Ok(match self {
            Renderer::Vulkan(r) => r.create_frame_buffers(render_pass)?,
            Renderer::None => Vec::new(),
        })
    }

    //#endregion

    //#region Render Pipeline

    pub fn create_simple_render_pipeline(
        &self,
        render_pass: &RenderPass,
        vs: shaders::simple::vs::Shader,
        fs: shaders::simple::fs::Shader,
    ) -> Result<RenderPipeline> {
        println!("Creating simple pipeline...");

        Ok(match self {
            Renderer::Vulkan(r) => r.create_simple_render_pipeline(render_pass, vs, fs)?,
            Renderer::None => RenderPipeline::None,
        })
    }

    //#endregion

    pub fn begin_frame(&mut self) {
        match self {
            Renderer::Vulkan(r) => r.begin_frame(),
            Renderer::None => (),
        }
    }

    // TODO: this should just take the command buffers to execute
    pub fn draw_data<F>(
        &mut self,
        _render_pipeline: &RenderPipeline,
        clear_values: [f32; 4],
        //draw_data: Vec<Arc<dyn BufferAccess + Send + Sync>>,
        frame_buffers: F,
    ) -> Result<bool>
    where
        F: AsRef<Vec<FrameBuffer>>,
    {
        let frame_buffers = frame_buffers.as_ref();

        Ok(match self {
            Renderer::Vulkan(r) => {
                let clear_values = vec![clear_values.into()];

                let acquire_future = r.acquire_swapchain()?;
                if acquire_future.is_none() {
                    return Ok(false);
                }
                let acquire_future = acquire_future.unwrap();

                let frame_buffer = &frame_buffers[r.get_current_swapchain_image()];

                let command_buffer = r
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
                    /*.draw(
                        render_pipeline.pipeline.clone(),
                        &self.dynamic_state,
                        draw_data,
                        (),
                        (),
                    )?*/
                    .end_render_pass()?
                    .build()?;

                r.submit(acquire_future, command_buffer)?
            }
            Renderer::None => true,
        })
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}
