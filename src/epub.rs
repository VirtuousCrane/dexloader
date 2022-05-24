//! This module concerns with how images are converted into
//! epub

extern crate epub_builder;

use std::fs::{self, File};
use crate::manga::MangaImage;
use epub_builder::{
    EpubBuilder,
    Result,
    ZipLibrary,
    EpubContent,
    ReferenceType,
    TocElement
};

static PAGE_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">

<head>
    <link rel="stylesheet" type="text/css" href="../Styles/page.css"/>
    <meta name="viewport" content = "width = 1600, height = 2400" />
</head>

<body>
    <div class="pagediv">
        <img src="IMAGE_SOURCE" class="bkgImg">
    </div>
</body>
</html>"#;

pub struct Book {
    pub constructor: EpubBuilder<ZipLibrary>,
    pub resources: Vec<BookContent>,
}

pub struct BookContent {
    pub order: i32,
    pub chapter_order: i32,
    pub target_path: String,
//    pub content: Vec<u8>,
    pub content: Content,
    pub content_type: String,
}

pub enum Content {
    Image(Vec<u8>),
    Text(String),
}

impl Book {
    pub fn new() -> Self {
        let constructor = EpubBuilder::new(ZipLibrary::new().unwrap())
            .unwrap();
        Book { constructor, resources: Vec::<BookContent>::new() }
    }

    pub fn add_author(&mut self, name: &str) -> Result<()> {
        self.constructor.metadata("author", name)?;
        Ok(())
    }

    pub fn add_title(&mut self, title: &str) -> Result<()> {
        self.constructor.metadata("title", title)?;
        Ok(())
    }

    pub fn add_css(&mut self, path: &str, target: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let css = fs::read(path)?;
        self.constructor.add_resource(target, css.as_slice(), "text/css")?;
        Ok(())
    }

    pub fn add_image(&mut self, img: &mut MangaImage, order: i32, target_path: &str, chapter_order: i32) {
        let img_free = std::mem::replace(&mut img.image, Vec::new());
        let resource = BookContent {
            order,
            chapter_order,
            target_path: String::from(target_path),
            content: Content::Image(img_free),
            content_type: String::from("image/jpeg")
        };
        self.resources.push(resource);
    }

    pub fn add_chapter_partition(&mut self, title: &str, target_path: &str) {
        let resource = BookContent {
            order: -1,
            chapter_order: -1,
            target_path: String::from(target_path),
            content: Content::Text(String::from(title)),
            content_type: String::from("chapter")
        };
        self.resources.push(resource);
    }

    pub fn add_cover_image(&mut self, img: &mut MangaImage) -> Result<()> {
        let image = std::mem::replace(&mut img.image, Vec::new());
        self.constructor.add_cover_image("Images/cover.jpg", image.as_slice(), "image/jpeg")?;
        Ok(())
    }

    pub fn generate(&mut self, output_path: &str) -> Result<()> {
        // TODO: USE output_path
        self.constructor.inline_toc();
        
        for resource in &self.resources {
            match &resource.content {
                Content::Image(data) => {
                    self.constructor.add_resource(&resource.target_path, data.as_slice(), "image/jpeg")?;
                    
                    let page_html = PAGE_TEMPLATE.replace(
                        "IMAGE_SOURCE",
                        &format!("../{}", &resource.target_path)
                    );
                    let content = EpubContent::new(
                        format!("Text/{}_{}.xhtml", &resource.chapter_order, &resource.order),
                        page_html.as_bytes()
                    )
                        .reftype(ReferenceType::Text);
                    self.constructor.add_content(content)?;
                },
                Content::Text(data) => {
                    let content = EpubContent::new(&resource.target_path, "".as_bytes())
                        .title(data)
                        .reftype(ReferenceType::Text);
                    
                    self.constructor.add_content(content)?;
                },
            }
        }

        let f = File::create(output_path)
            .expect(&format!("Failed to create file: {}", output_path));
        //self.constructor.generate(&mut std::io::stdout())?;
        self.constructor.generate(f)?;
        
        Ok(())
    }
}