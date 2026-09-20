use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize)]
pub struct ProfileRes {
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
pub struct BasicProfileRes {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct EditProfileReq {
    #[validate(length(min = 1, max = 16))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[serde(rename(deserialize = "firstName"))]
    pub first_name: Option<String>,
    #[serde(rename(deserialize = "lastName"))]
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}