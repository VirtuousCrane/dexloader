use reqwest;
use tokio::task::{self, JoinHandle};
use futures::future::join_all;
use image::DynamicImage;

async fn async_get_image(url: String) -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
    let res = reqwest::get(url)
        .await?
        .bytes()
        .await?;
    
    let image = image::load_from_memory(&res)?;
    Ok(image)
}

pub async fn async_get_image_batch(batch: &mut Vec<String>, concurrent_process : i32) -> Vec<DynamicImage> {
    let mut handle_vec: Vec<JoinHandle<Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();
    let mut index_count = 0;

    for i in 0..concurrent_process {
        match batch.pop() {
            Some(url) => {
                handle_vec.push(task::spawn(async_get_image(url)));
            },
            None => (),
        }
    }

    let result = join_all(handle_vec)
        .await
        .into_iter()
        .map(Result::unwrap)
        .map(Result::unwrap)
        .collect();
    
    result
}