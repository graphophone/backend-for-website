use serde::Serialize;

#[derive(Serialize)]
pub struct UserProfileResponse {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
    #[serde(rename(serialize = "bannerUrl"))]
    pub banner_url: Option<String>,
    #[serde(rename(serialize = "firstName"))]
    pub first_name: Option<String>,
    #[serde(rename(serialize = "lastName"))]
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct UserBasicProfileResponse {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
}