use crate::api::Order;
use crate::api::TimeRange;
use crate::db::DB404Err;
use crate::db::DBPostCommentErr;
use crate::db::DBPostLikeErr;
use crate::db::DBUser;
use crate::db::DBUserPost;
use crate::db::SurrealCheckUtils;
use crate::db::SurrealErrUtils;
use crate::db::SurrealSerializeUtils;
use crate::db::post::create_post_id;
use surrealdb::types::SurrealValue;
use surrealdb::types::ToSql;
use tracing::trace;

use super::Db;
pub use surrealdb::Connection;
use surrealdb::types::RecordId;
use surrealdb::types::RecordIdKey;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct DBPostComment {
    pub id: RecordId,
    pub user: DBUser,
    pub post: RecordId,
    pub parent: Vec<RecordId>,
    pub text: String,
    pub modified_at: u128,
    pub created_at: u128,
}

pub fn create_post_comment_id(id: impl Into<RecordIdKey>) -> RecordId {
    RecordId::new("post_comment", id.into())
}

impl<C: Connection> Db<C> {
    pub async fn add_post_comment(
        &self,
        time: u128,
        user_id: RecordId,
        post_id: impl Into<RecordIdKey>,
        post_comment_reply: Option<String>,
        text: impl Into<String>,
    ) -> Result<DBPostComment, DBPostCommentErr> {
        let post_id = post_id.into();
        let parent_id = post_comment_reply.map(|v| create_post_comment_id(v));
        let q = r#"
                 LET $post = SELECT id FROM ONLY $post_id;
                 CREATE post_comment SET
                    user = $user_id,
                    post = $post.id,
                    parent = if $parent_id {
                        LET $post = SELECT id, parent FROM ONLY $parent_id;
                        if $post {
                            if $post.parent { $post.parent } else { [] } + [$post.id]
                        } else {
                            []
                        }
                    } else {
                        []
                    },
                    text = $comment_text,
                    modified_at = $time,
                    created_at = $time
                 RETURN *, user.*;
                "#;
        trace!("about to run {q}");
        self.db
            .query(q)
            .bind(("time", time))
            .bind(("user_id", user_id))
            .bind(("post_id", create_post_id(post_id.clone())))
            .bind(("comment_text", text.into()))
            .bind(("parent_id", parent_id))
            .await
            .check_good(|err| match err {
                err if err.field_value_null("post_comment") => {
                    DBPostCommentErr::ReplyCommentNotFound(post_id.to_sql())
                }
                err if err.field_value_null("post") => {
                    DBPostCommentErr::PostNotFound(post_id.to_sql())
                }
                err => err.into(),
            })
            .and_then_take_expect(1)
    }

    pub async fn get_post_comments(
        &self,
        time: u128,
        post_id: impl Into<RecordIdKey>,
        parent_id: Option<impl Into<RecordIdKey>>,
        flatten: bool,
        limit: usize,
        time_range: TimeRange,
        order: Order,
    ) -> Result<Vec<DBPostComment>, DB404Err> {
        let parent_id = parent_id.map(|v| create_post_comment_id(v.into()));

        let q_time_after = match time_range {
            TimeRange::None => "",
            TimeRange::Less(_) => "AND created_at < $time_range",
            TimeRange::LessOrEqual(_) => "AND created_at <= $time_range",
            TimeRange::More(_) => "AND created_at > $time_range",
            TimeRange::MoreOrEqual(_) => "AND created_at >= $time_range",
        };
        let time_range = match time_range {
            TimeRange::None => 0,
            TimeRange::Less(v)
            | TimeRange::LessOrEqual(v)
            | TimeRange::More(v)
            | TimeRange::MoreOrEqual(v) => v,
        };
        let q_order = match order {
            Order::OneTwoThree => "ASC",
            Order::ThreeTwoOne => "DESC",
        };
        let q_parent = match (&parent_id, flatten) {
            (Some(_), true) => "AND parent.find($parent_id)",
            (Some(_), false) => "AND parent.last() = $parent_id",
            (None, true) => "",
            (None, false) => "AND parent.len() = 0",
        };

        // let mut q_computed = String::new();
        // q_computed.push_str(q_time_after);
        // q_computed.push_str(q_parent);
        // let q_comupted = q_computed.trim();


        let q = format!(
            "
            SELECT *, user.* FROM post_comment WHERE
                    post = $post_id
                    {q_time_after}
                    {q_parent}
                    ORDER BY created_at {q_order}
                    LIMIT $comment_limit
        "
        );
        trace!("about to run {q}");
        self.db
            .query(q)
            .bind(("time", time))
            .bind(("time_range", time_range))
            .bind(("parent_id", parent_id))
            .bind(("post_id", create_post_id(post_id.into())))
            .bind(("comment_limit", limit))
            .await
            .check_good(DB404Err::from)
            .and_then_take_all(0)
    }

    pub async fn delete_post_comment(
        &self,
        user_id: RecordId,
        comment_id: impl Into<RecordIdKey>,
    ) -> Result<(), surrealdb::Error> {
        let comment_id = create_post_comment_id(comment_id.into());
        let q = "DELETE post_comment WHERE (parent.find($comment_id) OR id == $comment_id) AND user = $user_id";
        trace!("about to run {q} with input $comment_id: {comment_id:?}, $user_id: {user_id:?}");
        self.db
            .query(q)
            .bind(("comment_id", comment_id))
            .bind(("user_id", user_id))
            .await
            .check_good(surrealdb::Error::from)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use surrealdb::{
        engine::local::Mem,
        types::{RecordId, ToSql},
    };
    use tracing::trace;

    use crate::{
        api::{ChangeUsernameErr, Order, ServerRes, TimeRange},
        db::{
            AddUserErr, DB404Err, DBChangeUsernameErr, DBPostLikeErr, DBSentEmailReason,
            DBUserPostFile, Db, DBEmailIsTakenErr, post_like::create_post_like_id,
        },
        init_test_log,
    };

    #[tokio::test]
    async fn db_post_comment_scroll_top_test() {
        init_test_log();
        crate::init_test_log();
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let post = db
            .add_post(0, "hey1", "title", "description", 0, vec![])
            .await
            .unwrap();

        db.add_post_comment(0, user.id.clone(), post.id.key.clone(), None, "wow1")
            .await
            .unwrap();

        db.add_post_comment(1, user.id.clone(), post.id.key.clone(), None, "wow2")
            .await
            .unwrap();

        db.add_post_comment(2, user.id.clone(), post.id.key.clone(), None, "wow3")
            .await
            .unwrap();

        db.add_post_comment(3, user.id.clone(), post.id.key.clone(), None, "wow4")
            .await
            .unwrap();

        db.add_post_comment(4, user.id.clone(), post.id.key.clone(), None, "wow5")
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                2,
                TimeRange::More(0),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow2", "wow3",]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                2,
                TimeRange::More(2),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow4", "wow5",]);
    }

    #[tokio::test]
    async fn db_post_comment_scroll_btm_test() {
        init_test_log();
        crate::init_test_log();
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let post = db
            .add_post(0, "hey1", "title", "description", 0, vec![])
            .await
            .unwrap();

        db.add_post_comment(0, user.id.clone(), post.id.key.clone(), None, "wow1")
            .await
            .unwrap();

        db.add_post_comment(1, user.id.clone(), post.id.key.clone(), None, "wow2")
            .await
            .unwrap();

        db.add_post_comment(2, user.id.clone(), post.id.key.clone(), None, "wow3")
            .await
            .unwrap();

        db.add_post_comment(3, user.id.clone(), post.id.key.clone(), None, "wow4")
            .await
            .unwrap();

        db.add_post_comment(4, user.id.clone(), post.id.key.clone(), None, "wow5")
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                2,
                TimeRange::Less(4),
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow4", "wow3",]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                2,
                TimeRange::Less(2),
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow2", "wow1",]);
    }

    #[tokio::test]
    async fn db_post_comment_ord_test() {
        init_test_log();
        crate::init_test_log();
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();
        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let post = db
            .add_post(0, "hey1", "title", "description", 0, vec![])
            .await
            .unwrap();

        db.add_post_comment(0, user.id.clone(), post.id.key.clone(), None, "wow1")
            .await
            .unwrap();

        db.add_post_comment(1, user.id.clone(), post.id.key.clone(), None, "wow2")
            .await
            .unwrap();

        db.add_post_comment(2, user.id.clone(), post.id.key.clone(), None, "wow3")
            .await
            .unwrap();

        db.add_post_comment(3, user.id.clone(), post.id.key.clone(), None, "wow4")
            .await
            .unwrap();

        db.add_post_comment(4, user.id.clone(), post.id.key.clone(), None, "wow5")
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                5,
                TimeRange::More(2),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow4", "wow5",]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                5,
                TimeRange::Less(2),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow1", "wow2"]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                5,
                TimeRange::MoreOrEqual(2),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow3", "wow4", "wow5",]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                5,
                TimeRange::LessOrEqual(2),
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow1", "wow2", "wow3"]);

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                5,
                TimeRange::LessOrEqual(2),
                Order::ThreeTwoOne,
            )
            .await
            .unwrap();

        let text = comments
            .into_iter()
            .map(|v| v.text)
            .collect::<Vec<String>>();
        assert_eq!(text, vec!["wow3", "wow2", "wow1"]);
    }

    #[tokio::test]
    async fn db_post_comment_test_one() {
        init_test_log();

        crate::init_test_log();
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();

        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let post = db
            .add_post(0, "hey1", "title", "description", 0, vec![])
            .await
            .unwrap();

        db.add_post_comment(0, user.id.clone(), post.id.key.clone(), None, "wow1")
            .await
            .unwrap();

        db.add_post_comment(1, user.id.clone(), post.id.key.clone(), None, "wow2")
            .await
            .unwrap();

        db.add_post_comment(2, user.id.clone(), post.id.key.clone(), None, "wow3")
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                3,
                post.id.key.clone(),
                None::<String>,
                false,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();

        let comment_first = comments.first().unwrap();

        db.delete_post_comment(user.id.clone(), comment_first.id.key.clone())
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                3,
                post.id.key.clone(),
                None::<String>,
                false,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();
        assert!(comments.len() == 2);
    }

    #[tokio::test]
    async fn db_post_comment_reply_test() {
        init_test_log();

        crate::init_test_log();
        let db = Db::new::<Mem>(()).await.unwrap();
        db.migrate(0).await.unwrap();

        let user = db.add_user(0, "hey1", "hey1@hey.com", "123").await.unwrap();
        let post = db
            .add_post(1, "hey1", "title", "description", 0, vec![])
            .await
            .unwrap();

        let post_comment = db
            .add_post_comment(2, user.id.clone(), post.id.key.clone(), None, "wow1")
            .await
            .unwrap();

        let post_reply = db
            .add_post_comment(
                3,
                user.id.clone(),
                post.id.key.clone(),
                Some(post_comment.id.key.clone().to_sql()),
                "wowza",
            )
            .await
            .unwrap();

        let post_reply2 = db
            .add_post_comment(
                4,
                user.id.clone(),
                post.id.key.clone(),
                Some(post_reply.id.key.clone().to_sql()),
                "wowza2",
            )
            .await
            .unwrap();

        let comments = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                false,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();
        assert!(comments.len() == 1);

        let comment_replies = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                Some(post_comment.id.key.clone().to_sql()),
                false,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();
        assert!(comment_replies.len() == 1);
        assert!(comment_replies[0].text == "wowza");


        let comment_replies = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                Some(post_comment.id.key.clone().to_sql()),
                true,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();
        assert!(comment_replies.len() == 2);
        assert!(comment_replies[0].text == "wowza");
        assert!(comment_replies[1].text == "wowza2");

        let comment_replies = db
            .get_post_comments(
                5,
                post.id.key.clone(),
                None::<String>,
                true,
                3,
                TimeRange::None,
                Order::OneTwoThree,
            )
            .await
            .unwrap();
        assert!(comment_replies.len() == 3);
        assert!(comment_replies[0].text == "wow1");
        assert!(comment_replies[1].text == "wowza");
        assert!(comment_replies[2].text == "wowza2");
    }
}
