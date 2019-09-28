pub mod shaders;
mod vulkan;

use std::fmt;
use std::sync::Arc;

use derivative::Derivative;
use failure::{bail, Error};
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::pipeline::GraphicsPipelineAbstract;
use winit::Window;

pub use vulkan::VulkanRendererState;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Derivative)]
#[derivative(Default)]
pub enum VertexBuffer {
    Vulkan(Arc<CpuAccessibleBuffer<[Vertex]>>),

    #[derivative(Default)]
    None,
}

impl fmt::Display for VertexBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VertexBuffer::Vulkan(_) => write!(f, "Vulkan"),
            VertexBuffer::None => write!(f, "None"),
        }
    }
}

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

    pub fn get_window(&self) -> Result<&Window> {
        Ok(match self {
            Renderer::Vulkan(r) => r.get_window(),
            Renderer::None => bail!("No window"),
        })
    }

    //#endregion

    fn init_viewport(&mut self) {
        match self {
            Renderer::Vulkan(r) => r.init_viewport(),
            Renderer::None => (),
        }
    }

    pub fn recreate_swapchain(&mut self) -> Result<bool> {
        Ok(match self {
            Renderer::Vulkan(r) => r.recreate_swapchain()?,
            Renderer::None => true,
        })
    }

    //#region CPU Buffers

    pub fn create_vertex_buffer<V>(&self, vertices: V) -> Result<VertexBuffer>
    where
        V: Into<Vec<Vertex>>,
    {
        Ok(match self {
            Renderer::Vulkan(r) => VertexBuffer::Vulkan(r.create_cpu_buffer_iter(vertices)?),
            Renderer::None => VertexBuffer::None,
        })
    }

    //#endregion

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
        render_pipeline: &RenderPipeline,
        clear_values: [f32; 4],
        draw_data: &VertexBuffer,
        frame_buffers: F,
    ) -> Result<bool>
    where
        F: AsRef<Vec<FrameBuffer>>,
    {
        Ok(match self {
            Renderer::Vulkan(r) => {
                r.draw_data(render_pipeline, clear_values, draw_data, frame_buffers)?
            }
            Renderer::None => false,
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
