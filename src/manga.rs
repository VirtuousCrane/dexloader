//! This module contains everything about a manga and its
//! information. This includes, but not limited to, the
//! manga itself, its chapters, the chapter's information
//! and so on.

extern crate serde;
extern crate async_trait;
extern crate image;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use async_trait::async_trait;
use image::DynamicImage;
use image::error::ImageResult;
use std::convert::From;

use crate::util;
use crate::connection;
use crate::epub::Book;

#[async_trait]
/// A **data structure** that can make asynchronous get requests
pub trait AsyncGet {
    /// Makes an asynchronous get request and parse the result in a json format.
    /// 
    /// The return format can be denoted by the generic.
    async fn async_get_json<T>(&self, url: &str) -> T 
        where T: DeserializeOwned
    {
        let body = reqwest::get(url)
            .await
            .expect("Error yeeting a request")
            .json::<T>()
            .await
            .expect("Failed deserializing json");
        
        body
    }
}

/// Contains information about the pagination of manga
/// chapter list.
#[derive(Serialize, Deserialize)]
struct Pagination {
    limit: i32,
    offset: i32,
    total: i32,
}

/// The list of manga chapters and its pagination information.
#[derive(Serialize, Deserialize)]
pub struct ChapterList {
    pub data: Vec<Chapter>,

    #[serde(flatten)]
    pagination: Pagination,
}

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

/// Contains the response from MangaDex's @Home server.
/// 
/// This information is required to construct a URL to
/// download the information required to fetch the manga's
/// image links.
#[derive(Serialize, Deserialize)]
struct AtHomeServerResponse {
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

/// A wrapper for DynamicImage with page number included.
pub struct MangaImage {
    pub page_no: i32,
    pub image: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct MangaData {
    pub data: MangaDataInner,
}

#[derive(Serialize, Deserialize)]
pub struct MangaDataInner {
    pub attributes: MangaTitle,
    pub relationships: Vec<MangaRelation>,
}

#[derive(Serialize, Deserialize)]
pub struct MangaTitle {
    pub title: MangaEnglishTitle,
}

#[derive(Serialize, Deserialize)]
pub struct MangaEnglishTitle {
    pub en: String,
}

#[derive(Serialize, Deserialize)]
pub struct MangaRelation {
    id: String,
    #[serde(rename = "type")]
    relation_type: String,
}

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

pub struct ChapterImage {
    pub chapter_no: f32,
    pub chapter_title: String,
    pub target_name: String,
    pub images: Vec<MangaImage>,
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

        req_url.push_str("&offset=");
        req_url.push_str(&offset.to_string());

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

impl MangaData {
    pub fn get_title(&self) -> &str {
        &self.data.attributes.title.en
    }

    pub fn get_author_id(&self) -> String {
        let relationships = &self.data.relationships;
        let mut id = String::from("");
        for relation in relationships.iter() {
            if relation.relation_type == "author" {
                id.push_str(&relation.id);
                break;
            }
        }

        return id;
    }

    pub fn get_cover_id(&self) -> String {
        let relationships = &self.data.relationships;
        let mut id = String::from("");
        for relation in relationships.iter() {
            if relation.relation_type == "cover_art" {
                id.push_str(&relation.id);
                break;
            }
        }

        return id;
    }
}

impl AuthorData {
    pub fn get_name(&self) -> &str {
        &self.data.attributes.name
    }
}

impl CoverData {
    pub fn get_file_name(&self) -> &str {
        &self.data.attributes.file_name
    }
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

impl MangaImage {
    /// Creates a MangaImage object from the page number
    /// of the image and the image itself.
    pub fn new(page_no: i32, image: Vec<u8>) -> Self {
        MangaImage { page_no, image }
    }

    /// Saves the image to a path
    pub fn save(&self, file_name: String) -> ImageResult<()> {
        let image = image::load_from_memory(&self.image)
            .expect("Failed to parse image from bytes");
        image.save(file_name)
    }
}