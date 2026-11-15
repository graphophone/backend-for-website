use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddCommentReq {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateCommentReq {
    pub new_content: String,
}
