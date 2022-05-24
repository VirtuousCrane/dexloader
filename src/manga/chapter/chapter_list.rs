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