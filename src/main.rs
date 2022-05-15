use std::io;

use dexloader::manga::Manga;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut url = String::new();

    io::stdin()
        .read_line(&mut url)
        .expect("Failed to read line");
    let url = url.trim();
    
    let mut manga = Manga::from(url);
    //manga.get_chapters(None, 0).await;
    //let c = &chapters.get_chapters()[0];
    //let chapters = &manga.chapter_list.unwrap();
    //let c = &chapters.data[0];
    manga.get_chapters(Some(1), 0).await;
    manga.download_chapters().await;
    manga.generate_epub();

    //c.download().await;

    Ok(())
}
