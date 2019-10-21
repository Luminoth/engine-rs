use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::actor::ActorAsset;
use super::resource::Resource;

#[derive(Serialize, Deserialize)]
pub(crate) struct PrefabAsset {
    pub id: Uuid,

    pub actor: ActorAsset,
}

impl Resource for PrefabAsset {
    const EXTENSION: &'static str = "prefab";
}
