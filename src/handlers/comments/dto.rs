use serde::{Deserialize, Serialize};

use crate::handlers;

#[derive(Deserialize)]
pub struct AddCommentReq {
    #[serde(rename(deserialize = "musicId"))]
    pub music_id: i64,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateCommentReq {
    #[serde(rename(deserialize = "newContent"))]
    pub new_content: String,
}

#[derive(Deserialize)]
pub struct CommentsPageQuery {
    #[serde(rename(deserialize = "pageNumber"))]
    pub page_number: i64,
}

#[derive(Serialize)]
pub struct MusicCommentsPage {
    #[serde(rename(serialize = "totalPages"))]
    pub total_pages: i64,
    pub comments: Vec<MusicCommentEntry>,
}

#[derive(Serialize)]
pub struct MusicCommentEntry {
    #[serde(rename(serialize = "commentId"))]
    pub comment_id: i64,
    pub profile: Option<handlers::user::dto::MyProfileRes>,
    pub content: String,
    #[serde(rename(serialize = "likesCount"))]
    pub likes_count: i64,
    #[serde(rename(serialize = "isLiked"))]
    pub is_liked: bool,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct UserCommentsPage {
    #[serde(rename(serialize = "totalPages"))]
    pub total_pages: i64,
    pub comments: Vec<UserCommentEntry>,
}

#[derive(Serialize)]
pub struct UserCommentEntry {
    #[serde(rename(serialize = "commentId"))]
    pub comment_id: i64,
    pub music: Option<handlers::music::dto::ShortMusicRes>,
    pub content: String,
    #[serde(rename(serialize = "likesCount"))]
    pub likes_count: i64,
    #[serde(rename(serialize = "isLiked"))]
    pub is_liked: bool,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: i64,
}
