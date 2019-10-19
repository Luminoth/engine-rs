use serde::{Deserialize, Serialize};
use specs::prelude::*;

use core::math::{Quaternion, Vector3};

use crate::assets::ComponentAsset;

#[derive(Component, Default, Debug, Serialize, Deserialize)]
pub struct TransformComponent {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

#[typetag::serde]
impl ComponentAsset for TransformComponent {}
