extern crate async_trait;

use async_trait::async_trait;
use std::convert::From;

use crate::epub::Book;
use crate::connection::{self, AsyncGet};

use super::manga_data::MangaData;
use super::author::AuthorData;
use super::cover::CoverData;
use super::chapter::{ChapterList, ChapterImage, MangaImage};

/// Contains the ID of the manga and its URL.
pub struct Manga {
    pub id: String,
    pub url: String,
    pub data: Option<MangaData>,
    pub title: String,
    pub chapter_list: Option<ChapterList>,
    pub chapter_images: Vec<ChapterImage>,
    pub author_name: String,
}

#[async_trait]
impl AsyncGet for Manga {}

impl Manga {
    /// Extracts a manga's id from its URL
    fn get_manga_id_from_url(url: & str) -> String {
        let mut splitted_url = url.split("/");
        String::from(splitted_url.nth(4).unwrap())
    }

    /// Construct the URL for fetching information about a manga.
    /// 
    /// The offset indicates the "page" number of the request, and
    /// the chapter limit dictates how many chapters of manga is to
    /// be fetched per "page".
    fn construct_manga_chapter_request_url(&self, offset: i32, chapter_limit: Option<i32>) -> String {
        let mut req_url = String::from("https://api.mangadex.org/chapter?manga=");
        req_url.push_str(&self.id);
        req_url.push_str("&translatedLanguage[]=en");

        match chapter_limit {
            Some(i) => {
                req_url.push_str("&limit=");
                req_url.push_str(&i.to_string())
            },
            None => (),
        };

        if offset != 0 {
            req_url.push_str("&offset=");
            req_url.push_str(&offset.to_string());
        }

        req_url
    }

    /// Gets the list of all chapters of a manga asynchronously.
    /// 
    /// The offset indicates the "page" number of the request,
    /// and the chapter limit dictates how many chapters of
    /// manga to be displayed per "page".
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
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_chapters(&mut self, chapter_limit: Option<i32>, offset: i32) {
        let request_url = self.construct_manga_chapter_request_url(offset, chapter_limit);
        self.chapter_list = Some(self.async_get_json::<ChapterList>(&request_url).await);
    }

    async fn get_manga_info(&mut self) -> MangaData {
        let request_url = format!("https://api.mangadex.org/manga/{}", &self.id);
        let data = self.async_get_json::<MangaData>(&request_url).await;

        data
    }

    pub async fn fetch_author(&mut self) {
        let author_id = self.data.as_ref().unwrap().get_author_id();
        let request_url = format!("https://api.mangadex.org/author/{}", author_id);
        let author_data = self.async_get_json::<AuthorData>(&request_url).await;
        let author_name = author_data.get_name();
        self.author_name.push_str(author_name);
    }

    pub async fn fetch_cover(&self) -> MangaImage {
        let cover_id = self.data.as_ref().unwrap().get_cover_id();
        let request_url = format!("https://api.mangadex.org/cover/{}", cover_id);
        let cover_data = self.async_get_json::<CoverData>(&request_url).await;
        let cover_file_name = cover_data.get_file_name();
        
        let request_url = format!(
            "https://uploads.mangadex.org/covers/{}/{}",
            &self.id,
            &cover_file_name
        );
        let cover_image = connection::async_get_image(request_url, 0, false).await.unwrap();
        cover_image
    }

    pub async fn download_chapters(&mut self, clear_previous: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Only fetch general data if not fetched
        if self.title == "" || self.author_name == "" {
            self.data = Some(self.get_manga_info().await);
            
            self.title.push_str(self.data.as_ref().unwrap().get_title());
            self.fetch_author().await;
        }

        // Clear previous chapter data
        if clear_previous {
            self.chapter_images.clear();
        }
        
        // Storing images into the vector
        for chapter in self.chapter_list.as_ref().unwrap().data.iter() {
            let chapter_images = chapter.download().await;
            let chapter_img = ChapterImage {
                chapter_no: chapter.get_chapter_number(),
                chapter_title: String::from(chapter.get_name()),
                target_name: chapter.generate_file_name(),
                images: chapter_images
            };

            self.chapter_images.push(chapter_img);
        }

        Ok(())
    }

    pub fn get_total(&self) -> Option<i32> {
        match &self.chapter_list {
            Some(c) => {
                Some(c.pagination.total)
            },
            None => None
        }
    }

    pub async fn generate_epub(&mut self, output_path: &str) {
        let mut book = Book::new();
        
        book.add_author(&self.author_name);
        book.add_title(&self.title);
        book.add_css("assets/page.css", "Styles/page.css");

        let mut cover_image = self.fetch_cover().await;
        book.add_cover_image(&mut cover_image);

        for (j, ci) in self.chapter_images.iter_mut().enumerate() {
            //book.add_chapter_partition(&ci.chapter_title, &format!("Text/Chapter_{}.xhtml", ci.chapter_no));
            for (i, img) in ci.images.iter_mut().enumerate() {
                let temp_path_name = format!("{}_{}", j, i);
                book.add_image(
                    img,
                    i as i32,
                    &ci.target_name.replace("ORDER", &temp_path_name),
                    j as i32,
                );
            }
        }

        book.generate(output_path)
            .expect("Failed to generate epub");
    }
}

impl From<&str> for Manga {
    fn from(url: &str) -> Self {
        let id = Manga::get_manga_id_from_url(url);
        Manga { 
            id,
            url: String::from(url),
            data: None,
            title: String::from(""),
            chapter_list: None,
            chapter_images: Vec::new(),
            author_name: String::from("")
        }
    }
}
