pub mod actor;
mod quaternion;
mod transform;
mod vector;

use std::sync::Arc;

use failure::Error;
use parking_lot::RwLock;
use winit::EventsLoop;

pub use quaternion::*;
pub use transform::*;
pub use vector::*;

use renderer::Renderer;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Engine {
    events_loop: EventsLoop,

    renderer: Arc<RwLock<renderer::VulkanRenderer>>,
    pipeline: Arc<renderer::VulkanPipeline>,
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
            renderer: Arc::new(RwLock::new(renderer)),
        })
    }

    pub fn get_renderer(&self) -> &Arc<RwLock<renderer::VulkanRenderer>> {
        &self.renderer
    }

    pub fn create_pipeline(&mut self) {
        self.pipeline = self
            .renderer
            .read()
            .create_simple_pipeline(render_pass.clone(), vs, fs)
            .unwrap_or_else(|e| panic!("Error creating pipeline: {}", e));
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Running...");

        let mut quit = false;
        let mut recreate_swapchain = false;
        loop {
            {
                let mut renderer = self.renderer.write();
                renderer.begin_frame();

                if recreate_swapchain {
                    recreate_swapchain = false;
                }

                if !renderer.acquire_swapchain()? {
                    recreate_swapchain = true;
                    continue;
                }
            }

            self.events_loop.poll_events(|event| match event {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested,
                    ..
                } => quit = true,
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Resized(_),
                    ..
                } => recreate_swapchain = true,
                _ => (),
            });

            if quit {
                break;
            }
        }

        Ok(())
    }
}
