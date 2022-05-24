extern crate image;
use image::ImageResult;

pub struct ChapterImage {
    pub chapter_no: f32,
    pub chapter_title: String,
    pub target_name: String,
    pub images: Vec<MangaImage>,
}

/// A wrapper for DynamicImage with page number included.
pub struct MangaImage {
    pub page_no: i32,
    pub image: Vec<u8>,
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