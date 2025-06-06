use crate::ctx::Ctx;
use crate::model::manager::ModelManager;
use modql::field::HasFields;
use modql::SIden;
use sea_query::{Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use crate::error::{Error, Result};

#[derive(Iden)]
pub enum CommonIden{
    Id,
}
pub trait DbBmc {
    const TABLE: &'static str;
    fn table_ref() -> TableRef{
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let(columns,sea_values) = fields.for_sea_insert();
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values).map_err(|e|Error::QueryError(e.to_string()))?
        .returning(Query::returning().columns([CommonIden::Id]));

    let (sql,values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_,(i64,),_>(&sql,values)
        .fetch_one(db)
        .await?;
    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::field_column_refs())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql,values) = query.build_sqlx(PostgresQueryBuilder);
    let entity = sqlx::query_as_with::<_,E,_>(&sql,values)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound{
            entity:MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::field_column_refs());

    let (sql,values) = query.build_sqlx(PostgresQueryBuilder);
    let entities = sqlx::query_as_with::<_,E,_>(&sql,values)
        .fetch_all(db)
        .await?;

    Ok(entities)
}

pub async fn update<MC, E>(
    _ctx: &Ctx,
    mm: &ModelManager,
    id: i64,
    data: E,
) -> Result<()>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();

    let fields = data.not_none_fields();
    let fields = fields.for_sea_update();
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql,values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql,values)
        .execute(db)
        .await?
        .rows_affected();

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();

    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql,values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql,values)
        .execute(db)
        .await?
        .rows_affected();

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
