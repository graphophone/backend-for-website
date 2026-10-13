use serde::{Deserialize, Serialize};

use crate::handlers::user::dto::BasicProfileRes;

#[derive(Deserialize)]
pub struct UploadTrackReq {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "categoriesIds"))]
    pub categories_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct UploadThumbnailReq {
    #[serde(rename(deserialize = "mimeType"))]
    pub mime_type: String,
}

#[derive(Serialize)]
pub struct UploadThumbnailRes {
    #[serde(rename(serialize = "uploadUrl"))]
    pub upload_url: String,
    #[serde(rename(serialize = "thumbnailId"))]
    pub thumbnail_id: String,
}

#[derive(Deserialize)]
pub struct NotifyUploadThumbnailReq {
    #[serde(rename(deserialize = "thumbnailId"))]
    pub thumbnail_id: String,
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
    #[serde(rename(serialize = "uploader"))]
    pub uploader: BasicProfileRes,
    pub categories: Vec<CategoryRes>,
    #[serde(rename(serialize = "uploadStatus"))]
    pub upload_status: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: i64,
}