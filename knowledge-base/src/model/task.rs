use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::manager::ModelManager;
use crate::error::{Result};
use serde::{Deserialize, Serialize};
use modql::field::Fields;
use sqlx::FromRow;
use tracing::instrument;

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub created_by: i64,
}

#[derive(Debug, Fields, Deserialize)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Debug, Fields, Deserialize)]
pub struct TaskForCreateInternal {
    pub title: String,
    pub created_by: i64,
}

#[derive(Debug, Fields, Deserialize)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}

impl TaskBmc {
    #[instrument]
    pub async fn create(
        ctx: &Ctx,
        mm: &ModelManager,
        task_c: TaskForCreateInternal,
    ) -> Result<Task> {
        let id = base::create::<Self, _>(ctx, mm, task_c).await?;
        Self::get(ctx, mm, id).await
    }

    #[instrument]
    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    #[instrument]
    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, mm).await
    }

    #[instrument]
    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: TaskForUpdate,
    ) -> Result<Task> {
        base::update::<Self, _>(ctx, mm, id, task_u).await?;
        Self::get(ctx, mm, id).await
    }

    #[instrument]
    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        let task = Self::get(ctx, mm, id).await?;
        base::delete::<Self>(ctx, mm, id).await?;
        Ok(task)
    }
}
