use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::component::ComponentAsset;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct ActorAsset {
    pub id: Uuid,

    // TODO: need to be able to override the prefab values
    #[serde(default)]
    pub prefab: Option<Uuid>,

    #[serde(default)]
    pub components: Vec<Box<dyn ComponentAsset>>,

    #[serde(default)]
    pub children: Vec<ActorAsset>,
}
