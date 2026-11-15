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
pub struct FollowsCount {
    #[serde(rename(serialize = "followersCount"))]
    pub followers_count: i64,
    #[serde(rename(serialize = "followingsCount"))]
    pub followings_count: i64,
    #[serde(rename(serialize = "isFollowed"))]
    pub is_followed: bool,
}

#[derive(Serialize)]
pub struct MyProfileRes {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct BasicProfileRes {
    #[serde(rename(serialize = "userId"))]
    pub user_id: i64,
    pub username: String,
    #[serde(rename(serialize = "avatarUrl"))]
    pub avatar_url: Option<String>,
    #[serde(rename(serialize = "followsCount"))]
    pub follows_count: Option<FollowsCount>,
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

#[derive(Deserialize)]
pub struct UploadImageReq {
    #[serde(rename(deserialize = "mimeType"))]
    pub mime_type: String,
}

#[derive(Serialize)]
pub struct UploadImageRes {
    #[serde(rename(serialize = "uploadUrl"))]
    pub upload_url: String,
    #[serde(rename(serialize = "assetId"))]
    pub image_id: String,
}

#[derive(Deserialize)]
pub struct NotifyUploadImageReq {
    #[serde(rename(deserialize = "assetId"))]
    pub image_id: String,
}

#[derive(Deserialize)]
pub struct ImageTypeQuery {
    #[serde(rename(deserialize = "imageType"))]
    pub image_type: String,
}

#[derive(Deserialize)]
pub struct FollowsPageQuery {
    #[serde(rename(deserialize = "pageNumber"))]
    pub page_number: i64,
}

#[derive(Serialize)]
pub struct FollowsPageRes {
    #[serde(rename(serialize = "totalPages"))]
    pub total_pages: i64,
    pub profiles: Vec<BasicProfileRes>,
}
