use reqwest;
use tokio::task::{self, JoinHandle};
use std::collections::HashMap;

async fn async_get(url:&str, n: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _res = reqwest::get(url)
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!("{}", n);
    Ok(())
}

pub async fn get_request_url(url: &'static str, concurrent_process : i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Declaring a vector to store the async processes' join handle in
    let mut req_vec : Vec<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();

    // Spawns n concurrent processes
    for i in 1..concurrent_process + 1 {
        req_vec.push(task::spawn(async_get(url, i)));
    }

    // Gets the result of all those processes
    for item in req_vec.into_iter() {
        item.await??;
    }

    Ok(())
}