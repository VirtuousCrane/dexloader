use serde::{Serialize, Deserialize};

use super::chapter::Chapter;

/// The list of manga chapters and its pagination information.
#[derive(Serialize, Deserialize)]
pub struct ChapterList {
    pub data: Vec<Chapter>,

    #[serde(flatten)]
    pub pagination: Pagination,
}

/// Contains information about the pagination of manga
/// chapter list.
#[derive(Serialize, Deserialize)]
pub struct Pagination {
    pub limit: i32,
    pub offset: i32,
    pub total: i32,
}

impl ChapterList {
    pub fn sort_chapters(&mut self) {
        self.data.sort_by(|chapter1, chapter2| chapter1.get_chapter_number().partial_cmp(&chapter2.get_chapter_number()).unwrap());
    }
}
