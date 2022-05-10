//! This module is concerned with making asynchronous
//! requests.

use reqwest;
use tokio::task::{self, JoinHandle};
use futures::future::join_all;

use crate::manga::MangaImage;

/// Makes a get request to fetch an image asynchronously
async fn async_get_image(url: String, page_no: i32) -> Result<MangaImage, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url)
        .await?
        .bytes()
        .await?;
    
    let image = image::load_from_memory(&res)?;
    let image = MangaImage::new(page_no, image);
    Ok(image)
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
                    handle_vec.push(task::spawn(async_get_image(url, page_no)));
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