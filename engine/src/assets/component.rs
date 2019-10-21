#[typetag::serde(tag = "type", content = "data")]
pub(crate) trait ComponentAsset: std::fmt::Debug {}
