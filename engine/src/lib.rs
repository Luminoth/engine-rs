pub mod actor;
mod quaternion;
mod transform;
mod vector;

use std::sync::Arc;

use failure::Error;
use winit::EventsLoop;

pub use quaternion::*;
pub use transform::*;
pub use vector::*;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Engine {
    events_loop: EventsLoop,

    renderer: Arc<renderer::VulkanRenderer>,
}

impl Engine {
    pub fn new<S>(appid: S) -> Result<Self>
    where
        S: Into<String>,
    {
        println!("Initializing engine...");

        let events_loop = EventsLoop::new();

        let renderer = renderer::VulkanRenderer::new(&events_loop)?;
        renderer.get_window().set_title(&appid.into());

        Ok(Self {
            events_loop,
            renderer: Arc::new(renderer),
        })
    }

    pub fn get_renderer(&self) -> &Arc<renderer::VulkanRenderer> {
        &self.renderer
    }

    pub fn run(&mut self) {
        println!("Running...");

        self.events_loop.run_forever(|event| match event {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::CloseRequested,
                ..
            } => winit::ControlFlow::Break,
            _ => winit::ControlFlow::Continue,
        });
    }
}
