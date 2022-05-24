use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CoverData {
    data: CoverDataInner
}

#[derive(Serialize, Deserialize)]
pub struct CoverDataInner {
    attributes: CoverAttribute
}

#[derive(Serialize, Deserialize)]
pub struct CoverAttribute {
    #[serde(rename = "fileName")]
    pub file_name: String
}

impl CoverData {
    pub fn get_file_name(&self) -> &str {
        &self.data.attributes.file_name
    }
}
