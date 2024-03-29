use serde::{Deserialize, Serialize};
use specs::prelude::*;

use crate::assets::ComponentAsset;

// TODO: requires a transform component
#[derive(Component, Debug, Serialize, Deserialize)]
pub struct CameraComponent {}

#[typetag::serde]
impl ComponentAsset for CameraComponent {}
