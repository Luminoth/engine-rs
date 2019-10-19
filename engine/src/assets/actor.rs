use serde::{Deserialize, Serialize};

use super::component::ComponentAsset;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct ActorAsset {
    // TODO: need to be able to override the prefab values
    #[serde(default)]
    pub prefab: Option<String>,

    #[serde(default)]
    pub components: Vec<Box<dyn ComponentAsset>>,

    #[serde(default)]
    pub children: Vec<ActorAsset>,
}
