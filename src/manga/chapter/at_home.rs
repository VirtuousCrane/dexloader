use serde::{Serialize, Deserialize};

/// Contains the response from MangaDex's @Home server.
/// 
/// This information is required to construct a URL to
/// download the information required to fetch the manga's
/// image links.
#[derive(Serialize, Deserialize)]
pub struct AtHomeServerResponse {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    
    #[serde(rename = "chapter")]
    pub chapter_data: ChapterData,
}

/// Contains the chapter hash and the image filenames.
#[derive(Serialize, Deserialize)]
pub struct ChapterData {
    pub hash: String,
    pub data: Vec<String>,

    #[serde(rename = "dataSaver")]
    pub data_saver: Vec<String>,
}

impl AtHomeServerResponse {
    /// Returns the chapter hash
    pub fn get_hash(&self) -> &str {
        self.chapter_data.hash.as_ref()
    }

    /// Returns a vector of manga image filenames.
    pub fn get_data(&self) -> &Vec<String> {
        self.chapter_data.data.as_ref()
    }

    /// Returns a mutable vector of manga image filenames.
    pub fn get_mut_data(&mut self) -> &mut Vec<String> {
        self.chapter_data.data.as_mut()
    }

    /// Returns a vector of manga image filenames with
    /// less resolution.
    pub fn get_data_save(&self) -> &Vec<String> {
        self.chapter_data.data_saver.as_ref()
    }

    /// Returns a mutable vector of manga image filenames
    /// with less resolution.
    pub fn get_mut_data_save(&mut self) -> &mut Vec<String> {
        self.chapter_data.data_saver.as_mut()
    }
}
