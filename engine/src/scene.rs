use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use log::{debug, info, warn};
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
        self.entities = World::new();

        let mut filepath = to_absolute_path(filepath)?;
        filepath.set_extension(SceneAsset::EXTENSION);
        info!("Loading scene from {}...", filepath.display());

        warn!("TODO: load scene async");
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let asset = serde_json::from_reader::<_, SceneAsset>(reader)?;

        debug!("Loading {} actors...", asset.actors.len());
        for actor in asset.actors.iter() {
            if let Some(prefab) = &actor.prefab {
                warn!("TODO: load prefab {}", prefab);
            }

            let builder = self.entities.create_entity();
            for component in actor.components.iter() {
                warn!("TODO: add component");
            }
            builder.build();

            warn!("actor");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn save<P>(&self, filepath: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let asset = SceneAsset::default();
        warn!("TODO: build the scene asset");

        warn!("TODO: save scene async");
        let file = File::open(filepath)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &asset)?;

        Ok(())
    }
}
