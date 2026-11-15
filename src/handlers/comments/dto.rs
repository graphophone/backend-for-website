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
pub struct CommentsPage {
    #[serde(rename(serialize = "total_pages"))]
    pub total_pages: i64,
    pub comments: Vec<CommentEntry>,
}

#[derive(Serialize)]
pub struct CommentEntry {
    #[serde(rename(serialize = "commentId"))]
    pub comment_id: i64,
    pub profile: Option<handlers::user::dto::MyProfileRes>,
    pub content: String,
    #[serde(rename(serialize = "createdAt"))]
    pub created_at: i64,
}
