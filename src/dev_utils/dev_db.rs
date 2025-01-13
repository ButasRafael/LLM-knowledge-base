use crate::ctx::Ctx;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;
use crate::config::config;
use crate::model::manager::ModelManager;
use crate::model::user::{User, UserBmc};

type Db = Pool<Postgres>;

const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@postgres:5432/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@postgres:5432/app_db";
// sql files
const SQL_RECREATE_DB_FILE_NAME: &str = "00-recreate-db.sql";
const SQL_DIR: &str = "sql/dev_initial";

const DEMO_PWD: &str = "welcome";

#[tracing::instrument]
pub async fn init_dev_db() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:<12} - init_dev_db()", "FOR-DEV-ONLY");

    let sql_recreate_db_file = format!("{}/{}", SQL_DIR, SQL_RECREATE_DB_FILE_NAME);
    let root_db = new_db_pool(PG_DEV_POSTGRES_URL).await?;
    pexec(&root_db, &sql_recreate_db_file).await?;

    // -- Get sql files.
    let mut paths: Vec<PathBuf> = fs::read_dir(SQL_DIR)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    paths.sort();

    // -- SQL Execute each file.
    let app_db = new_db_pool(PG_DEV_APP_URL).await?;

    for path in paths {
        if let Some(path_str) = path.to_str() {
            let path_normalized = path_str.replace("\\", "/");

            if path_normalized.ends_with(".sql") && !path_normalized.ends_with(SQL_RECREATE_DB_FILE_NAME) {
                pexec(&app_db, &path_normalized).await?;
            }
        }
    }

    let config = config();
    let mm = ModelManager::new(config).await?;
    let ctx = Ctx::root_ctx();
    let admin_user:User = UserBmc::first_by_username(&ctx, &mm, "admin").await?.unwrap();
    UserBmc::update_pwd(&ctx, &mm, admin_user.id, DEMO_PWD).await?;


    info!("Successfully initialized dev db.");

    Ok(())
}


#[tracing::instrument]
async fn pexec(db: &Db, file: &str) -> Result<(), sqlx::Error> {
    println!("{:<12} - pexec: {file:?}", "FOR-DEV-ONLY");

    // -- Read the file.
    let content = fs::read_to_string(file)?;
    
    let sqls: Vec<&str> = content.split(';').collect();

    for sql in sqls {
        sqlx::query(sql).execute(db).await.map_err(|e| {
            println!("pexec error while running:\n{sql}");
            println!("cause:\n{e}");
            e
        })?;
    }
    info!("Successfully executed {file}.");

    Ok(())
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .connect(db_con_url)
        .await
}