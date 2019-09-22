pub struct Entity {
    id: String,
}

impl Entity {
    pub fn new<S>(id: S) -> Self
    where
        S: Into<String>,
    {
        Self { id: id.into() }
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }
}
