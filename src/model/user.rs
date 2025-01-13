use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields, SqlBuilder, Whereable};
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use uuid::Uuid;
use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::manager::ModelManager;
use crate::error::{Error, Result};
use tracing::instrument;
use crate::crypt::{pwd, EncryptContent};

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
    pub role: Option<String>,
}

#[derive(Debug,Fields, Deserialize)]
pub struct UserForUpdate {
    pub username: Option<String>,
}
#[derive(Debug,Fields, Deserialize)]
struct UserForInsert {
    username: String,
    role: String,
}

#[derive(Debug,Clone,Fields, FromRow)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,
    pub pwd: Option<String>,
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
    pub role: String,
}
#[derive(Debug,Clone,Fields, FromRow)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,
    pub token_salt: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordChange {
    pub old_password: String,
    pub new_password: String,
}


pub trait UserBy:HasFields + for<'r> FromRow <'r, PgRow> + Unpin + Send{
}

impl UserBy for User{}

impl UserBy for UserForLogin{}
impl UserBy for UserForAuth{}




pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    #[instrument]
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        user_c: UserForCreate,
    ) -> Result<i64> {
        let role = user_c.role.unwrap_or_else(|| "user".to_string());
        let user_insert = UserForInsert {
            username: user_c.username.clone(),
            role,
        };
        let new_user_id = base::create::<Self, _>(ctx, mm, user_insert).await?;

        Self::update_pwd(ctx, mm, new_user_id, &user_c.pwd_clear).await?;

        Ok(new_user_id)
    }

    #[instrument]
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where E: UserBy
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn first_by_username<E>(_ctx: &Ctx, mm: &ModelManager, username: &str) -> Result<Option<E>>
    where E: UserBy
    {
        let db = mm.db();
        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional::<_,E>(db)
            .await?;
        Ok(user)
    }

    pub async fn update_pwd(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        pwd_clear: &str,
    ) -> Result<()> {
        let db = mm.db();
        let user:UserForLogin = Self::get(ctx,mm,id).await?;
        let pwd = pwd::encrypt_pwd(&EncryptContent {
            content: pwd_clear.to_string(),
            salt: user.pwd_salt.to_string(),
        })?;

        sqlb::update()
            .table(Self::TABLE)
            .and_where("id", "=", id)
            .data(vec![("pwd", pwd.to_string()).into()])
            .exec(db)
            .await?;
        Ok(())

    }


    #[instrument]
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<User>> {
        base::list::<Self, _>(ctx, mm).await
    }

    #[instrument]
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        user_u: UserForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, user_u).await
    }

    #[instrument]
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }

    #[instrument]
    pub async fn get_by_username(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<User> {
        let db = mm.db();

        let user: Option<User> = sqlx::query_as::<_, User>(
            "SELECT id, username FROM \"user\" WHERE username = $1",
        )
            .bind(username)
            .fetch_optional(db)
            .await?;

        user.ok_or(Error::EntityNotFound {
            entity: "user",
            id: 0,
        })
        }



}
