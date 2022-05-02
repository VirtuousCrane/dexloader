use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use async_trait::async_trait;
use image::{self, DynamicImage};
use image::error::ImageResult;

use crate::util;
use crate::connection;

#[async_trait]
pub trait AsyncGet {
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

#[derive(Serialize, Deserialize)]
struct Pagination {
    limit: i32,
    offset: i32,
    total: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ChapterList {
    data: Vec<Chapter>,

    #[serde(flatten)]
    pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub struct Chapter {
    id: String,
    attributes: ChapterAttribute
}

#[derive(Serialize, Deserialize)]
struct ChapterAttribute {
    #[serde(rename = "chapter")]
    #[serde(deserialize_with = "util::deserialize_to_f32")]
    no: f32,
    pages: i32,
    title: String
}

#[derive(Serialize, Deserialize)]
struct AtHomeServerResponse {
    #[serde(rename = "baseUrl")]
    base_url: String,
    
    #[serde(rename = "chapter")]
    chapter_data: ChapterData,
}

#[derive(Serialize, Deserialize)]
struct ChapterData {
    hash: String,
    data: Vec<String>,

    #[serde(rename = "dataSaver")]
    data_saver: Vec<String>,
}

#[derive(Debug)]
pub struct Manga {
    id: String,
    url: String,
}

pub struct MangaImage {
    page_no: i32,
    image: DynamicImage,
}

#[async_trait]
impl AsyncGet for Manga {}

impl Manga {
    pub fn from(url: &str) -> Self {
        let id = Manga::get_manga_id_from_url(url);
        Manga { id, url : String::from(url) }
    }

    fn get_manga_id_from_url(url: & str) -> String {
        let mut splitted_url = url.split("/");
        String::from(splitted_url.nth(4).unwrap())
    }

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

    pub async fn get_chapters(&self, chapter_limit: Option<i32>, offset: i32) -> ChapterList {
        let request_url = self.construct_manga_chapter_request_url(offset, chapter_limit);
        let body = self.async_get_json::<ChapterList>(&request_url).await;

        body
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
}

impl ChapterList {
    pub fn get_chapters(&self) -> &Vec<Chapter> {
        &self.data
    }
}

#[async_trait]
impl AsyncGet for Chapter {}

impl Chapter {
    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_chapter_number(&self) -> f32 {
        self.attributes.no
    }

    pub fn get_pages(&self) -> i32 {
        self.attributes.pages
    }

    pub fn get_name(&self) -> &str {
        &self.attributes.title
    }

    async fn get_manga_at_home_info(&self) -> AtHomeServerResponse {
        let mut url = String::from("https://api.mangadex.org/at-home/server/");
        url.push_str(self.get_id());

        let body = self.async_get_json::<AtHomeServerResponse>(&url).await;
        body
    }

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
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }
    
    pub fn get_hash(&self) -> &str {
        &self.chapter_data.hash
    }

    pub fn get_data(&self) -> &Vec<String> {
        &self.chapter_data.data
    }

    pub fn get_mut_data(&mut self) -> &mut Vec<String> {
        &mut self.chapter_data.data
    }

    pub fn get_data_save(&self) -> &Vec<String> {
        &self.chapter_data.data_saver
    }

    pub fn get_mut_data_save(&mut self) -> &mut Vec<String> {
        &mut self.chapter_data.data_saver
    }
}

impl MangaImage {
    pub fn from(page_no: i32, image: DynamicImage) -> MangaImage {
        MangaImage { page_no, image }
    }

    pub fn save(&self, file_name: String) -> ImageResult<()> {
        self.image.save(file_name)
    }

    pub fn get_page_no(&self) -> i32 {
        self.page_no
    }
}