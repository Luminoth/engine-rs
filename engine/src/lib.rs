mod actor;
pub mod components;

use chrono::prelude::*;
use failure::Error;
use winit::EventsLoop;

pub use actor::*;

pub type Result<T> = std::result::Result<T, Error>;

// https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/triangle.rs

pub enum RendererType {
    Vulkan,
}

struct EngineStats {
    frame_count: u64,
    start_time: DateTime<Utc>,
    last_fps_dump: DateTime<Utc>,
}

impl EngineStats {
    pub fn fps(&self) -> f32 {
        let duration = (Utc::now() - self.start_time).num_seconds();
        if duration == 0 {
            0.0
        } else {
            self.frame_count as f32 / duration as f32
        }
    }
}

impl Default for EngineStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            start_time: Utc::now(),
            last_fps_dump: Utc::now(),
        }
    }
}

#[derive(Default)]
struct EngineDebug {
    enable_debug_window: bool,
}

pub struct Engine {
    events_loop: EventsLoop,

    renderer: renderer::Renderer,
    render_pass: renderer::RenderPass,
    frame_buffers: Vec<renderer::FrameBuffer>,
    render_pipeline: renderer::RenderPipeline,

    stats: EngineStats,
    debug: EngineDebug,
}

impl Engine {
    pub fn new<S>(appid: S, renderer_type: RendererType) -> Result<Self>
    where
        S: Into<String>,
    {
        println!("Initializing engine...");

        let events_loop = EventsLoop::new();

        let renderer = match renderer_type {
            RendererType::Vulkan => {
                renderer::Renderer::Vulkan(renderer::VulkanRendererState::new(&events_loop)?)
            }
        };
        renderer.set_window_title(&appid.into());

        Ok(Self {
            events_loop,
            renderer,
            render_pass: renderer::RenderPass::None,
            frame_buffers: Vec::new(),
            render_pipeline: renderer::RenderPipeline::None,
            stats: EngineStats::default(),
            debug: EngineDebug::default(),
        })
    }

    pub fn load_scene(&mut self) -> Result<()> {
        println!("Loading scene...");

        // TODO: setup the vertex buffer

        let (vs, fs) = self
            .renderer
            .load_simple_shader()
            .unwrap_or_else(|e| panic!("Error loading simple shader: {}", e));

        self.render_pass = self
            .renderer
            .create_simple_render_pass()
            .unwrap_or_else(|e| panic!("Error creating render pass: {}", e));

        self.render_pipeline = self
            .renderer
            .create_simple_render_pipeline(&self.render_pass, vs, fs)
            .unwrap_or_else(|e| panic!("Error creating render pipeline: {}", e));

        self.frame_buffers = self
            .renderer
            .create_frame_buffers(&self.render_pass)
            .unwrap_or_else(|e| panic!("Error creating frame buffers: {}", e));

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        println!("Running...");

        let mut quit = false;
        let mut recreate_swapchain = false;
        loop {
            let frame_start = Utc::now();

            self.renderer.begin_frame();

            if recreate_swapchain {
                // TODO: recreate the swapchain

                recreate_swapchain = false;
            }

            if !self.renderer.draw_data(
                &self.render_pipeline,
                [0.0, 0.0, 1.0, 1.0],
                &self.frame_buffers,
            )? {
                recreate_swapchain = true;
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

            self.stats.frame_count += 1;

            let now = Utc::now();
            let frame_time = now - frame_start;

            // TODO:  average FPS vs last frame time extrapolated
            // also print the frame time
            if (now - self.stats.last_fps_dump).num_seconds() >= 1 {
                println!(
                    "Render Stats:
\tFrames: {}
\tFrame Time: {}ms
\tFPS: {}",
                    self.stats.frame_count,
                    frame_time.num_milliseconds(),
                    self.stats.fps()
                );
                self.stats.last_fps_dump = now.clone();
            }

            if quit {
                break;
            }
        }

        Ok(())
    }
}
