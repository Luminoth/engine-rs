mod actor;
pub mod components;
mod scene;

use chrono::prelude::*;
use failure::Error;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::EventsLoop;

pub use actor::*;
use scene::*;

pub type Result<T> = std::result::Result<T, Error>;

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

struct EngineDebug {
    imgui: imgui::Context,
    imgui_platform: WinitPlatform,
    enable_debug_window: bool,
}

impl Default for EngineDebug {
    fn default() -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let imgui_platform = WinitPlatform::init(&mut imgui);

        let hidpi_factor = imgui_platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    ..imgui::FontConfig::default()
                }),
            }]);
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        Self {
            imgui,
            imgui_platform,
            enable_debug_window: false,
        }
    }
}

pub struct Engine {
    events_loop: EventsLoop,

    renderer: renderer::Renderer,
    render_pass: renderer::RenderPass,
    frame_buffers: Vec<renderer::FrameBuffer>,
    render_pipeline: renderer::RenderPipeline,

    scene: Scene,

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
        renderer.get_window()?.set_title(&appid.into());

        let mut engine = Self {
            events_loop,
            renderer,
            render_pass: renderer::RenderPass::None,
            frame_buffers: Vec::new(),
            render_pipeline: renderer::RenderPipeline::None,

            scene: Scene::default(),

            stats: EngineStats::default(),
            debug: EngineDebug::default(),
        };

        engine.debug.imgui_platform.attach_window(
            engine.debug.imgui.io_mut(),
            engine.renderer.get_window()?,
            HiDpiMode::Default,
        );

        Ok(engine)
    }

    pub fn load_scene(&mut self) -> Result<()> {
        println!("Loading scene...");

        self.scene.vertex_buffer = self.renderer.create_vertex_buffer(vec![
            renderer::Vertex {
                position: [-0.5, -0.25, 0.0],
            },
            renderer::Vertex {
                position: [0.0, 0.5, 0.0],
            },
            renderer::Vertex {
                position: [0.25, -0.1, 0.0],
            },
        ])?;

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
        let mut _last_frame = std::time::Instant::now();

        loop {
            let frame_start = Utc::now();

            self.renderer.begin_frame();

            self.events_loop.poll_events(|event| {
                /*self.renderer
                .get_window()
                .and_then(|window| {
                    self.debug.imgui_platform.handle_event(
                        self.debug.imgui.io_mut(),
                        self.renderer.get_window()?,
                        &event,
                    );

                    Ok(())
                })
                .unwrap_or_else(|e| {
                    eprintln!("No window");
                });*/

                match event {
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::CloseRequested,
                        ..
                    } => quit = true,
                    winit::Event::WindowEvent {
                        event: winit::WindowEvent::Resized(_),
                        ..
                    } => recreate_swapchain = true,
                    _ => (),
                }
            });

            if recreate_swapchain {
                if !self.renderer.recreate_swapchain()? {
                    continue;
                }

                self.frame_buffers = self.renderer.create_frame_buffers(&self.render_pass)?;

                recreate_swapchain = false;
            }

            /*self.debug
                .imgui_platform
                .prepare_frame(self.debug.imgui.io_mut(), self.renderer.get_window()?)
                .unwrap_or_else(|e| eprintln!("{}", e));
            last_frame = self.debug.imgui.io_mut().update_delta_time(last_frame);

            let ui = self.debug.imgui.frame();*/

            if !self.renderer.draw_data(
                &self.render_pipeline,
                [0.0, 0.0, 1.0, 1.0],
                &self.scene.vertex_buffer,
                &self.frame_buffers,
            )? {
                recreate_swapchain = true;
            }

            /*self.debug
                .imgui_platform
                .prepare_render(&ui, self.renderer.get_window()?);
            // TODO: render debug data
            let _draw_data = ui.render();*/

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
