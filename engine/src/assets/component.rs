#[typetag::serde(tag = "type", content = "data")]
pub(crate) trait ComponentAsset: std::fmt::Debug {
    /*type Component;

    fn create_component(&self) -> Self::Component;*/
}
