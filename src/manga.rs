pub mod manga;
pub use manga::Manga;

pub mod author;
pub use author::AuthorData;

pub mod cover;
pub use cover::CoverData;

pub mod manga_data;
pub use manga_data::MangaData;

pub mod chapter;
pub use chapter::{Chapter, MangaImage, ChapterImage, ChapterList, AtHomeServerResponse};