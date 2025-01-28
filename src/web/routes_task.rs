
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{delete, get, post, put};
use tracing::info;
use crate::ctx::Ctx;
use crate::model::task::{Task, TaskForCreate, TaskForUpdate, TaskBmc, TaskForCreateInternal};
use crate::model::manager::ModelManager;

use crate::Result;
#[tracing::instrument]
async fn create_task(
    State(mm): State<ModelManager>,
    ctx:Ctx,
    Json(task_fc): Json<TaskForCreate>,
) -> Result<Json<Task>> {
    println!("->> {:<12} - create_task", "HANDLER");
    let task_internal = TaskForCreateInternal {
        title: task_fc.title,
        created_by: ctx.user_id(),
    };
    let task = TaskBmc::create(&ctx, &mm, task_internal).await?;
    info!("Task created: {:?}", task);
    Ok(Json(task))
}
#[tracing::instrument]
async fn list_tasks(
    State(mc): State<ModelManager>,
    ctx:Ctx,
) -> Result<Json<Vec<Task>>> {
    println!("->> {:<12} - list_tasks", "HANDLER");
    let tasks = TaskBmc::list(&ctx, &mc).await?;
    info!("Tasks listed");
    Ok(Json(tasks))
}
#[tracing::instrument]
async fn delete_task(
    State(mc): State<ModelManager>,
    ctx:Ctx,
    Path(id): Path<i64>,
) -> Result<Json<Task>> {
    println!("->> {:<12} - delete_task", "HANDLER");
    let task = TaskBmc::delete(&ctx, &mc, id).await?;
    info!("Task deleted: {:?}", task);
    Ok(Json(task))
}
#[tracing::instrument]
async fn get_task(
    State(mc): State<ModelManager>,
    ctx: Ctx,
    Path(id): Path<i64>,
) -> Result<Json<Task>> {
    println!("->> {:<12} - get_task", "HANDLER");
    let task = TaskBmc::get(&ctx, &mc, id).await?;
    info!("Task get: {:?}", task);
    Ok(Json(task))
}
#[tracing::instrument]
async fn update_task(
    State(mc): State<ModelManager>,
    ctx:Ctx,
    Path(id): Path<i64>,
    Json(task_fc): Json<TaskForUpdate>,
) -> Result<Json<Task>> {
    println!("->> {:<12} - update_task", "HANDLER");
    let task = TaskBmc::update(&ctx, &mc, id, task_fc).await?;
    info!("Task updated: {:?}", task);
    Ok(Json(task))
}

pub fn routes(mm:ModelManager)->Router{
    Router::new()
        .route("/tasks",post(create_task))
        .route("/tasks",get(list_tasks))
        .route("/tasks/:id",delete(delete_task))
        .route("/tasks/:id",get(get_task))
        .route("/tasks/:id",put(update_task))
        .with_state(mm)
}
