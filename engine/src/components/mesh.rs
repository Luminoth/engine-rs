use serde::{Deserialize, Serialize};
use specs::prelude::*;

use crate::assets::ComponentAsset;

#[derive(Component, Debug, Serialize, Deserialize)]
pub struct MeshComponent {
    //pub vertex_buffer: renderer::VertexBuffer,
}

#[typetag::serde]
impl ComponentAsset for MeshComponent {}
