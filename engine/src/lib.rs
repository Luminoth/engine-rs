mod actor;
pub mod components;
pub mod config;
mod scene;

#[macro_use]
extern crate specs_derive;

use chrono::prelude::*;
use failure::Error;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{Event, EventsLoop, Window};

pub use actor::*;
use scene::*;

pub type Result<T> = std::result::Result<T, Error>;

pub enum RendererType {
    Vulkan,
}

struct EngineStats {
    frame_count: u64,
    start_time: DateTime<Utc>,
    last_frame_start: DateTime<Utc>,
    last_fps_dump: DateTime<Utc>,
}

impl EngineStats {
    fn frame_time(&self) -> i64 {
        (Utc::now() - self.last_frame_start).num_milliseconds()
    }

    fn fps(&self) -> f32 {
        let duration = (Utc::now() - self.start_time).num_seconds();
        if duration == 0 {
            0.0
        } else {
            self.frame_count as f32 / duration as f32
        }
    }

    fn log_frame_stats(&mut self) {
        let now = Utc::now();
        if (now - self.last_fps_dump).num_seconds() < 5 {
            return;
        }

        // TODO:  average FPS vs last frame time extrapolated
        // also print the frame time
        println!(
            "Render Stats:
\tFrames: {}
\tFrame Time: {}ms
\tFPS: {}",
            self.frame_count,
            self.frame_time(),
            self.fps()
        );
        self.last_fps_dump = now;
    }
}

impl Default for EngineStats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            start_time: Utc::now(),
            last_frame_start: Utc::now(),
            last_fps_dump: Utc::now(),
        }
    }
}

struct EngineDebug {
    enable_debug_window: bool,

    imgui: imgui::Context,
    imgui_platform: WinitPlatform,
    last_frame: std::time::Instant,
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
            last_frame: std::time::Instant::now(),
        }
    }
}

impl EngineDebug {
    fn attach_window(&mut self, window: &Window) {
        self.imgui_platform
            .attach_window(self.imgui.io_mut(), window, HiDpiMode::Default);
    }

    #[allow(dead_code)]
    fn handle_event(&mut self, window: &Window, event: &Event) {
        self.imgui_platform
            .handle_event(self.imgui.io_mut(), window, event);
    }

    #[allow(dead_code)]
    fn prepare_frame(&mut self, window: &Window) {
        self.imgui_platform
            .prepare_frame(self.imgui.io_mut(), window)
            .unwrap_or_else(|e| eprintln!("{}", e));
        self.last_frame = self.imgui.io_mut().update_delta_time(self.last_frame);
    }

    #[allow(dead_code)]
    fn prepare_render(&self, ui: &imgui::Ui, window: &Window) {
        self.imgui_platform.prepare_render(ui, window);
    }
}

pub struct Engine {
    quit: bool,

    events_loop: EventsLoop,

    renderer: renderer::Renderer,
    render_pass: renderer::RenderPass,
    frame_buffers: Vec<renderer::FrameBuffer>,
    render_pipeline: renderer::RenderPipeline,
    recreate_swapchain: bool,

    scene: Scene,

    stats: EngineStats,
    debug: EngineDebug,
}

impl Engine {
    pub fn new<S>(
        appid: S,
        renderer_type: RendererType,
        window_config: &config::WindowConfig,
    ) -> Result<Self>
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

        println!("Resizing window {:?}", window_config);
        let window = renderer.get_window()?;
        window.set_title(&appid.into());
        window.set_inner_size(winit::dpi::LogicalSize::new(
            window_config.width.into(),
            window_config.height.into(),
        ));
        if window_config.fullscreen {
            window.set_fullscreen(Some(window.get_current_monitor()));
        }

        let mut engine = Self {
            quit: false,

            events_loop,

            renderer,
            render_pass: renderer::RenderPass::None,
            frame_buffers: Vec::new(),
            render_pipeline: renderer::RenderPipeline::None,
            recreate_swapchain: false,

            scene: Scene::default(),

            stats: EngineStats::default(),
            debug: EngineDebug::default(),
        };

        engine.debug.attach_window(engine.renderer.get_window()?);

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

        loop {
            self.stats.last_frame_start = Utc::now();

            self.renderer.begin_frame();

            self.handle_events()?;

            if self.recreate_swapchain {
                if !self.renderer.recreate_swapchain()? {
                    continue;
                }

                self.frame_buffers = self.renderer.create_frame_buffers(&self.render_pass)?;

                self.recreate_swapchain = false;
            }

            /*self.debug.prepare_frame(self.renderer.get_window()?);
            let _ui = self.debug.imgui.frame();*/

            self.render_scene()?;

            /*self.debug.prepare_render(&ui, self.renderer.get_window()?);

            // TODO: render debug data

            let _draw_data = ui.render();*/

            self.stats.frame_count += 1;
            self.stats.log_frame_stats();

            if self.quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        let window = self.renderer.get_window()?;
        let debug = &mut self.debug;

        let mut quit = false;
        let mut recreate_swapchain = false;

        self.events_loop.poll_events(|event| {
            debug.handle_event(window, &event);

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

        self.quit = quit;
        self.recreate_swapchain = recreate_swapchain;

        Ok(())
    }

    fn render_scene(&mut self) -> Result<()> {
        if !self.renderer.draw_data(
            &self.render_pipeline,
            [0.0, 0.0, 1.0, 1.0],
            &self.scene.vertex_buffer,
            &self.frame_buffers,
        )? {
            self.recreate_swapchain = true;
            return Ok(());
        }

        Ok(())
    }
}
