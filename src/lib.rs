//! # dexloader
//! 
//! The dexloader crate is a program and a library to download chapters of a manga
//! asynchronously from MangaDex.
//! 
//! While it is not yet implemented, we also plan to add the option to convert the downloaded manga into an epub
//! format as well.
//! 
//! # Example
//! ```
//! use dexloader::manga::Manga;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let url = "https://mangadex.org/title/259dfd8a-f06a-4825-8fa6-a2dcd7274230/yofukashi-no-uta";
//!     let manga = Manga::from(url);
//!     let chapters = manga.get_chapters(None, 0).await;
//!     let first_chapter = &chapters.data[0];
//! 
//!     first_chapter.download().await;
//! }
//! ```

pub mod connection;
pub mod manga;
pub mod util;