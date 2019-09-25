pub mod shaders;
mod vulkan;

use failure::Error;

pub use vulkan::*;

pub type Result<T> = std::result::Result<T, Error>;

pub trait RenderPass {}

pub trait Pipeline {}

pub trait Renderer {
    fn begin_frame(&mut self);
}

#[derive(Default, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}
vulkano::impl_vertex!(Vertex, position);

#[derive(Default, Copy, Clone)]
pub struct Triangle {
    pub vertices: [Vertex; 3],
}
