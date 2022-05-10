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
    data: Vec<Chapter>,

    #[serde(flatten)]
    pagination: Pagination,
}

/// A wrapper for the id of the manga chapter and its attributes
#[derive(Serialize, Deserialize)]
pub struct Chapter {
    id: String,
    attributes: ChapterAttribute
}

/// The attributes of a manga. Contains the chapter number, 
/// the number of pages, and the title of the chapter.
#[derive(Serialize, Deserialize)]
struct ChapterAttribute {
    #[serde(rename = "chapter")]
    #[serde(deserialize_with = "util::deserialize_to_f32")]
    no: f32,
    pages: i32,
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
    base_url: String,
    
    #[serde(rename = "chapter")]
    chapter_data: ChapterData,
}

/// Contains the chapter hash and the image filenames.
#[derive(Serialize, Deserialize)]
struct ChapterData {
    hash: String,
    data: Vec<String>,

    #[serde(rename = "dataSaver")]
    data_saver: Vec<String>,
}

/// Contains the ID of the manga and its URL.
#[derive(Debug)]
pub struct Manga {
    id: String,
    url: String,
}

/// A wrapper for DynamicImage with page number included.
pub struct MangaImage {
    page_no: i32,
    image: DynamicImage,
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
    /// let url = "https://mangadex.org/title/efb4278c-a761-406b-9d69-19603c5e4c8b/the-100-girlfriends-who-really-really-really-really-really-love-you";
    /// let manga = Manga::from(url);
    /// let chapters = manga.get_chapters(None, 0).await;
    /// ```
    pub async fn get_chapters(&self, chapter_limit: Option<i32>, offset: i32) -> ChapterList {
        let request_url = self.construct_manga_chapter_request_url(offset, chapter_limit);
        let body = self.async_get_json::<ChapterList>(&request_url).await;

        body
    }

    /// Returns the url of the manga
    pub fn get_url(&self) -> &str {
        &self.url
    }
}

impl From<&str> for Manga {
    fn from(url: &str) -> Self {
        let id = Manga::get_manga_id_from_url(url);
        Manga { id, url: String::from(url) }
    }
}

impl ChapterList {
    /// Gets all the chapter of a manga.
    pub fn get_chapters(&self) -> &Vec<Chapter> {
        &self.data
    }
}

#[async_trait]
impl AsyncGet for Chapter {}

impl Chapter {
    /// Returns the id of a chapter
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Returns the chapter number
    pub fn get_chapter_number(&self) -> f32 {
        self.attributes.no
    }

    /// Returns the number of pages of a chapter
    pub fn get_pages(&self) -> i32 {
        self.attributes.pages
    }

    /// Returns the chapter name of the chapter
    pub fn get_name(&self) -> &str {
        &self.attributes.title
    }

    /// Sends a request to the mangadex@home server asynchronously
    async fn get_manga_at_home_info(&self) -> AtHomeServerResponse {
        let mut url = String::from("https://api.mangadex.org/at-home/server/");
        url.push_str(self.get_id());

        let body = self.async_get_json::<AtHomeServerResponse>(&url).await;
        body
    }

    /// Downloads the images of a chapter of manga
    /// 
    /// # Examples
    /// ```
    /// let url = "https://mangadex.org/title/efb4278c-a761-406b-9d69-19603c5e4c8b/the-100-girlfriends-who-really-really-really-really-really-love-you";
    /// let manga = Manga::from(url);
    /// let chapters = manga.get_chapters(None, 0).await;
    /// let first_chapter = &chapters.get_chapters()[0];
    /// 
    /// first_chapter.download().await;
    /// ```
    pub async fn download(&self) {
        let mut at_home_info = self.get_manga_at_home_info().await;
        let mut url = String::from(at_home_info.get_base_url());
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
        
        // TODO: Remove this
        // Saving the images
        for (i, pic) in result.iter().enumerate() {
            let file_name = String::from("test_img/") + &i.to_string() + ".jpg";
            pic.save(file_name)
                .expect("Failed to save image");
        }
    }
}

impl AtHomeServerResponse {
    /// Returns the base url of a manga chapter
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Returns the chapter hash
    pub fn get_hash(&self) -> &str {
        &self.chapter_data.hash
    }

    /// Returns a vector of manga image filenames.
    pub fn get_data(&self) -> &Vec<String> {
        &self.chapter_data.data
    }

    /// Returns a mutable vector of manga image filenames.
    pub fn get_mut_data(&mut self) -> &mut Vec<String> {
        &mut self.chapter_data.data
    }

    /// Returns a vector of manga image filenames with
    /// less resolution.
    pub fn get_data_save(&self) -> &Vec<String> {
        &self.chapter_data.data_saver
    }

    /// Returns a mutable vector of manga image filenames
    /// with less resolution.
    pub fn get_mut_data_save(&mut self) -> &mut Vec<String> {
        &mut self.chapter_data.data_saver
    }
}

impl MangaImage {
    /// Creates a MangaImage object from the page number
    /// of the image and the image itself.
    pub fn from(page_no: i32, image: DynamicImage) -> MangaImage {
        MangaImage { page_no, image }
    }

    /// Saves the image to a path
    pub fn save(&self, file_name: String) -> ImageResult<()> {
        self.image.save(file_name)
    }

    /// Returns the page number of this image
    pub fn get_page_no(&self) -> i32 {
        self.page_no
    }
}