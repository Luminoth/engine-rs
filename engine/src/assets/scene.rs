use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::actor::ActorAsset;
use super::resource::Resource;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct SceneAsset {
    pub id: Uuid,

    #[serde(default)]
    pub actors: Vec<ActorAsset>,
}

impl Resource for SceneAsset {
    const EXTENSION: &'static str = "scene";
}
