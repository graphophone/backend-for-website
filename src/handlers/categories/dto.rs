use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(rename(deserialize = "searchToken"))]
    pub search_token: String,
}

#[derive(Serialize)]
pub struct CategoryData {
    pub id: i64,
    pub name: String,
}