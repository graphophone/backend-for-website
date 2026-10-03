use serde::Deserialize;

#[derive(Deserialize)]
pub struct UploadTrackReq {
    pub name: String,
    pub description: Option<String>,
    pub categories_ids: Vec<i64>,
}
