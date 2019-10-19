use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "viewer")]
pub struct Options {
    #[structopt(short, long)]
    scene: String,

    #[structopt(short, long, default_value = "1024")]
    width: u32,

    #[structopt(short, long, default_value = "768")]
    height: u32,

    #[structopt(short, long)]
    fullscreen: bool,
}

impl Options {
    pub fn get_scene(&self) -> &String {
        &self.scene
    }

    pub fn get_window_config(&self) -> engine::config::WindowConfig {
        engine::config::WindowConfig {
            width: self.width,
            height: self.height,
            fullscreen: self.fullscreen,
        }
    }
}
