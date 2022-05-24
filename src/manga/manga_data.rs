use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MangaData {
    pub data: MangaDataInner,
}

#[derive(Serialize, Deserialize)]
pub struct MangaDataInner {
    pub attributes: MangaTitle,
    pub relationships: Vec<MangaRelation>,
}

#[derive(Serialize, Deserialize)]
pub struct MangaTitle {
    pub title: MangaEnglishTitle,
}

#[derive(Serialize, Deserialize)]
pub struct MangaEnglishTitle {
    pub en: String,
}

#[derive(Serialize, Deserialize)]
pub struct MangaRelation {
    id: String,
    #[serde(rename = "type")]
    relation_type: String,
}

impl MangaData {
    pub fn get_title(&self) -> &str {
        &self.data.attributes.title.en
    }

    pub fn get_author_id(&self) -> String {
        let relationships = &self.data.relationships;
        let mut id = String::from("");
        for relation in relationships.iter() {
            if relation.relation_type == "author" {
                id.push_str(&relation.id);
                break;
            }
        }

        return id;
    }

    pub fn get_cover_id(&self) -> String {
        let relationships = &self.data.relationships;
        let mut id = String::from("");
        for relation in relationships.iter() {
            if relation.relation_type == "cover_art" {
                id.push_str(&relation.id);
                break;
            }
        }

        return id;
    }
}
