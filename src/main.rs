use std::io;

use dexloader::manga::Manga;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut url = String::new();

    io::stdin()
        .read_line(&mut url)
        .expect("Failed to read line");
    let url = url.trim();
    
    let manga = Manga::from(url);
    let chapters = manga.get_chapters(None, 0).await;
    //let c = &chapters.get_chapters()[0];
    let c = &chapters.data[0];

    c.download().await;

    Ok(())
}
