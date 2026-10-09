use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UploadTrackReq {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "categoriesIds"))]
    pub categories_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct CategoryRes {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct FullTrackRes {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename(serialize = "thumbnailUrl"))]
    pub thumbnail_url: Option<String>,
    #[serde(rename(serialize = "durationSeconds"))]
    pub duration_seconds: Option<i64>,
    #[serde(rename(serialize = "playCount"))]
    pub play_count: i64,
    #[serde(rename(serialize = "likeCount"))]
    pub like_count: i64,
    #[serde(rename(serialize = "uploaderId"))]
    pub uploader_id: i64,
    pub categories: Vec<CategoryRes>,
}