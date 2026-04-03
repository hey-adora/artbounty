use crate::api::User;

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
    pub user: User,
    pub post_key: String,
    pub parent_key: Vec<String>,
    pub text: String,
    pub replies_count: usize,
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

// #[derive(
//     Debug,
//     Default,
//     Clone,
//     PartialEq,
//     PartialOrd,
//     serde::Serialize,
//     serde::Deserialize,
//     rkyv::Archive,
//     rkyv::Serialize,
//     rkyv::Deserialize,
//     strum::EnumString,
//     strum::Display,
//     strum::EnumIter,
//     strum::EnumIs,
// )]
// pub enum CommentOrPostKey {
//     PostKey(String),
//     CommentKey(String),
// }

#[cfg(feature = "ssr")]
impl From<crate::db::post_comment::DBPostComment> for UserPostComment {
    fn from(value: crate::db::post_comment::DBPostComment) -> Self {
        use surrealdb::types::ToSql;

        Self {
            key: value.id.key.to_sql(),
            user: value.user.into(),
            post_key: value.post.key.to_sql(),
            parent_key: value.parent.into_iter().map(|v| v.key.to_sql()).collect(),
            text: value.text,
            replies_count: value.replies_count,
            modified_at: value.modified_at,
            created_at: value.created_at,
        }
    }
}
