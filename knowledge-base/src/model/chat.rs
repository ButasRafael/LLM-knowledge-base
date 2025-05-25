//! src/model/chat.rs
//! conversation + message data-access helpers

use chrono::{DateTime, Utc};                    // ← brings `chrono` into scope
use modql::field::{Fields, HasFields};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow};

use crate::{
    model::{
        base,
        base::{CommonIden, DbBmc},
    },
    Ctx, Result,
};


/* ────────────────────────────────────────────────────────────────────────── */
/*  Data structures                                                          */
/* ────────────────────────────────────────────────────────────────────────── */

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Conversation {
    pub id: i64,
    pub owner_id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Fields)]
pub struct ConversationForInsert {
    pub owner_id: i64,
    pub title: String,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub sender: String,
    pub content: String,
    pub token_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Fields)]
pub struct MessageForInsert {
    pub conversation_id: i64,
    pub sender: String,
    pub content: String,
    pub token_count: i32,
}

/* ────────────────────────────────────────────────────────────────────────── */
/*  BMC wrappers                                                             */
/* ────────────────────────────────────────────────────────────────────────── */

pub struct ConversationBmc;
impl DbBmc for ConversationBmc {
    const TABLE: &'static str = "conversation";
}

pub struct MessageBmc;
impl DbBmc for MessageBmc {
    const TABLE: &'static str = "message";
}

/* ────────────────────────────────────────────────────────────────────────── */
/*  Convenience helpers                                                      */
/* ────────────────────────────────────────────────────────────────────────── */

impl ConversationBmc {

    pub async fn get<E>(
        ctx: &Ctx,
        mm: &crate::model::manager::ModelManager,
        id: i64,
    ) -> Result<E>
    where
        E: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasFields,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }
    pub async fn create(
        ctx: &Ctx,
        mm: &crate::model::manager::ModelManager,
        title: &str,
    ) -> Result<i64> {
        base::create::<Self, _>(
            ctx,
            mm,
            ConversationForInsert {
                owner_id: ctx.user_id(),
                title: title.to_owned(),
            },
        )
            .await
    }

    /// Most-recent first, only conversations owned by the current user.
    pub async fn list_for_user(
        ctx: &Ctx,
        mm: &crate::model::manager::ModelManager,
    ) -> Result<Vec<Conversation>> {
        use sea_query::{Alias, Expr, Order, PostgresQueryBuilder, Query};
        use sea_query_binder::SqlxBinder;

        let mut q = Query::select();
        q.from(Self::table_ref())
            .columns(Conversation::field_column_refs())
            .and_where(Expr::col(Alias::new("owner_id")).eq(ctx.user_id()))
            .order_by(CommonIden::Id, Order::Desc);

        let (sql, values) = q.build_sqlx(PostgresQueryBuilder);
        let convs = sqlx::query_as_with::<_, Conversation, _>(&sql, values)
            .fetch_all(mm.db())
            .await?;
        Ok(convs)
    }
}

impl MessageBmc {
    pub async fn add(
        ctx: &Ctx,
        mm: &crate::model::manager::ModelManager,
        conv_id: i64,
        sender: &str,
        content: &str,
        token_count: i32,
    ) -> Result<i64> {
        base::create::<Self, _>(
            ctx,
            mm,
            MessageForInsert {
                conversation_id: conv_id,
                sender: sender.into(),
                content: content.into(),
                token_count,
            },
        )
            .await
    }

    /// newest → oldest, but only while the running token sum ≤ `limit`
    pub async fn recent(
        ctx: &Ctx,
        mm: &crate::model::manager::ModelManager,
        conv_id: i64,
        limit: i32,
    ) -> Result<Vec<Message>> {
        use sea_query::{Alias, Expr, Order, PostgresQueryBuilder, Query};
        use sea_query_binder::SqlxBinder;

        let mut q = Query::select();
        q.from(Self::table_ref())
            .columns(Message::field_column_refs())
            .and_where(Expr::col(Alias::new("conversation_id")).eq(conv_id))
            .order_by(CommonIden::Id, Order::Desc);

        let (sql, v) = q.build_sqlx(PostgresQueryBuilder);
        let mut msgs = sqlx::query_as_with::<_, Message, _>(&sql, v)
            .fetch_all(mm.db())
            .await?;

        // keep messages until token budget is exhausted
        let mut sum = 0;
        msgs.retain(|m| {
            sum += m.token_count;
            sum <= limit
        });
        Ok(msgs)
    }
}
