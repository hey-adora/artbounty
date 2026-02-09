#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct UserPostComment {
    pub key: String,
    pub user_key: String,
    pub post_key: String,
    pub comment_reply_key: Option<String>,
    pub text: String,
    pub modified_at: u128,
    pub created_at: u128,
}

#[derive(
    thiserror::Error,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum PostCommentErr {
    #[error("post \"{0}\" was not found")]
    PostNotFound(String),

    #[error("reply_comment \"{0}\" was not found")]
    ReplyCommentNotFound(String),
}

#[cfg(feature = "ssr")]
impl From<crate::db::comment::DBPostComment> for UserPostComment {
    fn from(value: crate::db::comment::DBPostComment) -> Self {
        Self {
            key: value.id.key().to_string(),
            user_key: value.user.key().to_string(),
            post_key: value.post.key().to_string(),
            comment_reply_key: value.comment.map(|v| v.key().to_string()),
            text: value.text,
            modified_at: value.modified_at,
            created_at: value.created_at,
        }
    }
}
