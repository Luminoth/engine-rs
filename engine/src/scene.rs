use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use log::{debug, info, warn};
use specs::prelude::*;

use core::fs::to_absolute_path;

use crate::assets::{Resource, SceneAsset};

#[derive(Default)]
pub struct Scene {
    entities: Vec<Entity>,
}

impl Scene {
    pub fn load<P>(&mut self, world: &mut World, filepath: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
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

            let builder = world.create_entity();
            for component in actor.components.iter() {
                warn!("TODO: add component");
            }
            self.entities.push(builder.build());

            warn!("actor");
        }

        Ok(())
    }

    pub fn unload(&mut self, world: &mut World) -> anyhow::Result<()> {
        for entity in self.entities.drain(0..) {
            world.delete_entity(entity)?;
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
