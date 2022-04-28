use mangadex_downloader::connection::get_request_url;

#[tokio::main]
async fn main() {
    let _ = get_request_url("https://httpbin.org/ip", 5).await;
}
