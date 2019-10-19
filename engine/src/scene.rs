use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use log::{info, warn};
use specs::prelude::*;

use core::fs::to_absolute_path;

use crate::assets::{Resource, SceneAsset};

pub struct Scene {
    entities: World,
}

impl Scene {
    pub fn new() -> Self {
        let entities = World::new();

        Self { entities }
    }

    pub fn load<P>(&mut self, filepath: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        warn!("TODO: unload current scene");
        let entities = World::new();

        let mut filepath = to_absolute_path(filepath)?;
        filepath.set_extension(SceneAsset::EXTENSION);
        info!("Loading scene from {}...", filepath.display());

        warn!("TODO: load scene async");
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let asset = serde_json::from_reader::<_, SceneAsset>(reader)?;

        for actor in asset.actors.iter() {
            warn!("actor");
        }

        Ok(())
    }
}
