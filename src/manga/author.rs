use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AuthorData {
    data: AuthorDataInner,
}

#[derive(Serialize, Deserialize)]
pub struct AuthorDataInner {
    attributes: AuthorAttribute
}

#[derive(Serialize, Deserialize)]
pub struct AuthorAttribute {
    name: String
}

impl AuthorData {
    pub fn get_name(&self) -> &str {
        &self.data.attributes.name
    }
}
