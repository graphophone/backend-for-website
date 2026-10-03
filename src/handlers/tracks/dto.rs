use serde::Deserialize;

#[derive(Deserialize)]
pub struct UploadTrackReq {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename(deserialize = "categoriesIds"))]
    pub categories_ids: Vec<i64>,
}
