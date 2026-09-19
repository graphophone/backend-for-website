use serde::Serialize;

#[derive(Serialize)]
pub struct UserProfileResponse {
    pub user_id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}