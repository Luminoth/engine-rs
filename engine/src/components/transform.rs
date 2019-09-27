use core::math::{Quaternion, Vector3};

#[derive(Default, Debug)]
pub struct TransformComponent {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl ecs::Component for TransformComponent {}
