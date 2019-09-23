use crate::quaternion::*;
use crate::vector::*;

pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}
