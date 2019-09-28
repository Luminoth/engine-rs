mod actor;
pub mod components;

use std::sync::Arc;

use failure::Error;
use parking_lot::RwLock;
use winit::EventsLoop;

pub use actor::*;
use renderer::Renderer;

pub type Result<T> = std::result::Result<T, Error>;

// https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/triangle.rs

pub struct Engine {
    events_loop: EventsLoop,

    renderer: Arc<RwLock<renderer::VulkanRenderer>>,
    render_pass: Option<renderer::VulkanRenderPass>,
    frame_buffers: Option<Vec<renderer::VulkanFrameBuffer>>,
    render_pipeline: Option<renderer::VulkanRenderPipeline>,
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
            render_pass: None,
            frame_buffers: None,
            render_pipeline: None,
        })
    }

    pub fn load_scene(&mut self) -> Result<()> {
        println!("Loading scene...");

        // TODO: setup the vertex buffer

        let (vs, fs) = self
            .renderer
            .read()
            .load_simple_shader()
            .unwrap_or_else(|e| panic!("Error loading simple shader: {}", e));

        self.render_pass = Some(
            self.renderer
                .read()
                .create_simple_render_pass()
                .unwrap_or_else(|e| panic!("Error creating render pass: {}", e)),
        );

        self.render_pipeline = Some(
            self.renderer
                .read()
                .create_simple_render_pipeline(self.render_pass.as_ref().unwrap(), vs, fs)
                .unwrap_or_else(|e| panic!("Error creating render pipeline: {}", e)),
        );

        self.frame_buffers = Some(
            self.renderer
                .write()
                .create_frame_buffers(self.render_pass.as_ref().unwrap())
                .unwrap_or_else(|e| panic!("Error creating frame buffers: {}", e)),
        );

        Ok(())
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
                    // TODO: recreate the swapchain

                    recreate_swapchain = false;
                }

                if !renderer.draw_data(
                    self.render_pipeline.as_ref().unwrap(),
                    vec![[0.0, 0.0, 1.0, 1.0].into()],
                    self.frame_buffers.as_ref().unwrap(),
                )? {
                    recreate_swapchain = true;
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
