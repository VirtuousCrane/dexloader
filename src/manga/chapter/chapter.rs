extern crate async_trait;

use serde::{Serialize, Deserialize};
use async_trait::async_trait;

use crate::util;
use crate::connection::{self, AsyncGet};

use super::at_home::AtHomeServerResponse;
use super::manga_image::MangaImage;

/// A wrapper for the id of the manga chapter and its attributes
#[derive(Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    attributes: ChapterAttribute
}

/// The attributes of a manga. Contains the chapter number, 
/// the number of pages, and the title of the chapter.
#[derive(Serialize, Deserialize)]
struct ChapterAttribute {
    //#[serde(rename = "chapter")]
    #[serde(deserialize_with = "util::deserialize_to_option_f32")]
    chapter: Option<f32>,
    pages: i32,

    #[serde(deserialize_with = "util::deserialize_title")]
    title: String
}

#[async_trait]
impl AsyncGet for Chapter {}

impl Chapter {
    /// Returns the chapter number
    pub fn get_chapter_number(&self) -> f32 {
        self.attributes.chapter.unwrap_or(0.0)
    }

    /// Returns the volume and chapter number to be used as file name
    pub fn generate_file_name(&self) -> String {
        String::from(format!("Images/{}_ORDER.jpg", self.attributes.chapter.unwrap_or(0.0)))
    }

    /// Returns the number of pages of a chapter
    pub fn get_pages(&self) -> i32 {
        self.attributes.pages
    }

    /// Returns the chapter name of the chapter
    pub fn get_name(&self) -> &str {
        self.attributes.title.as_ref()
    }

    /// Sends a request to the mangadex@home server asynchronously
    async fn get_manga_at_home_info(&self) -> AtHomeServerResponse {
        let mut url = String::from("https://api.mangadex.org/at-home/server/");
        url.push_str(&self.id);

        let body = self.async_get_json::<AtHomeServerResponse>(&url).await;
        body
    }

    /// Downloads the images of a chapter of manga
    /// 
    /// # Examples
    /// ```
    /// use dexloader::manga::Manga;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let url = "https://mangadex.org/title/efb4278c-a761-406b-9d69-19603c5e4c8b/the-100-girlfriends-who-really-really-really-really-really-love-you";
    ///     let manga = Manga::from(url);
    ///     let chapters = manga.get_chapters(None, 0).await;
    ///     let first_chapter = &chapters.data[0];
    /// 
    ///     first_chapter.download().await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn download(&self) -> Vec<MangaImage> {
        let mut at_home_info = self.get_manga_at_home_info().await;
        let mut url = String::from(&at_home_info.base_url);
        url.push_str("/data/");
        url.push_str(at_home_info.get_hash());
        url.push_str("/");

        let all_img = at_home_info.get_mut_data();
        let mut all_img_url: Vec<String> = all_img.iter()
            .map(|img_name| -> String {
                let mut u = String::from(&url);
                u.push_str(img_name);
                u
            })
            .collect();
        all_img_url.reverse();

        let result = connection::async_get_image_batch(&mut all_img_url, 5)
            .await;
        
        result
    }
}
