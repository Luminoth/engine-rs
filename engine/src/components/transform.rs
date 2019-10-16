use specs::prelude::*;

use core::math::{Quaternion, Vector3};

#[derive(Component, Default, Debug)]
pub struct TransformComponent {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}
