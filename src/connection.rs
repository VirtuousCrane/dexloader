//! This module is concerned with making asynchronous
//! requests.
extern crate reqwest;
extern crate async_trait;
extern crate tokio;
extern crate futures;
extern crate serde;

use tokio::task::{self, JoinHandle};
use futures::future::join_all;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use async_trait::async_trait;

use std::time::Instant;

use crate::manga::MangaImage;

/// A struct for containing the report POST request
#[derive(Serialize, Deserialize)]
struct ResponseBody {
    pub url: String,
    pub success: bool,
    pub cached: bool,
    pub bytes: usize,
    pub duration: u128
}


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

/// Makes a get request to fetch an image asynchronously
pub async fn async_get_image(url: String, page_no: i32, report: bool) -> Result<MangaImage, Box<dyn std::error::Error + Send + Sync>> {
    let url_clone = url.clone();

    let start_time = Instant::now();
    let res = reqwest::get(url)
        .await?;
    let elapsed_time = start_time.elapsed().as_millis();

    let headers = res.headers();
    let cache = match headers.get("x-cache") {
        Some(hv) => {hv.to_str().unwrap_or("MISS")},
        None => "MISS",
    };

    let mut cache_state = false; 
    if cache == "HIT" {
        cache_state = true;
    }

    let content_length = headers.get("content-length")
        .unwrap()
        .to_str()
        .unwrap_or("0");
    let content_length = content_length.parse::<usize>().unwrap();

    if !res.status().is_success() && report {
        async_report(url_clone, false, cache_state, content_length, elapsed_time)
            .await?;
        panic!("Failed to retrieve image!"); // TODO: Use ERR or retry instead
    }

    let image_bytes = res.bytes().await?;
    let image = image_bytes.to_vec();
    //let image = image::load_from_memory(&image_bytes)?;
    let image = MangaImage::new(page_no, image);

    if report {
        async_report(url_clone, true, cache_state, content_length, elapsed_time).await?;
    }
    
    Ok(image)
}

/// Reports back to the MangaDex@Home server
/// TODO: FIX CAPTCHA. CAPTCHA reply not yet working
async fn async_report(url: String, success: bool, cached: bool, bytes: usize, duration: u128) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let body = ResponseBody {url, success, cached, bytes, duration};

    let res = client.post("https://api.mangadex.network/report")
        .json(&body)
        .send()
        .await?;
    
    if res.status().as_u16() == 412 {
        let res_headers = res.headers();
        let captcha_ans = res_headers.get("X-Captcha-Sitekey")
            .unwrap()
            .to_str()
            .expect("Cannot get captcha answer");
        
        let cap_res = client.post("https://api.mangadex.network/report")
            .header("X-Captcha-Result", captcha_ans)
            .json(&body)
            .send()
            .await?;
    }

    Ok(())
}

/// Makes multiple get requests to fetch multiple images
/// asynchronously.
/// 
/// The number of images to download at the same time can
/// be specified with the "concurrent_process" variable.
pub async fn async_get_image_batch(batch: &mut Vec<String>, concurrent_process : i32) -> Vec<MangaImage> {
    let mut handle_vec: Vec<JoinHandle<Result<MangaImage, Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();
    let mut result_images: Vec<MangaImage> = Vec::new();

    // Iterating through all the images
    for i in (0..batch.len()).step_by(5) {
        // Download 5 images as a time
        for j in 0..concurrent_process {
            match batch.pop() {
                Some(url) => {
                    let page_no : i32 = i as i32 + j;
                    handle_vec.push(task::spawn(async_get_image(url, page_no, false)));
                },
                None => (),
            }
        }

        // Getting the result
        let mut result: Vec<MangaImage>  = join_all(handle_vec)
            .await
            .into_iter()
            .map(Result::unwrap)
            .map(Result::unwrap)
            .collect();

        result_images.append(&mut result);
        handle_vec = Vec::new();
    }
        
    result_images
}
