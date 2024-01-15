use sqlx::{PgPool, Postgres};
use uuid::Uuid;
use crate::models::errors::SResult;

pub trait Repo<'a, T>
where T: serde::Serialize + serde::Deserialize<'a> {
    async fn migrate(con_str: String) -> SResult<()>;
    async fn get(con_str: String, id: Uuid) -> SResult<T>;
    async fn get_all(con_str: String ) -> SResult<Vec<T>>;
    async fn post(con_str: String, model: &T) ->  SResult<()>;
    async fn put(con_str: String, id: Uuid, model: &T) -> SResult<()>;
    async fn delete(con_str: String, id: Uuid) -> SResult<()>;
}

pub async fn connect(con_str: String) -> SResult<sqlx::pool::PoolConnection<Postgres>> {
    let connection = PgPool::connect(&con_str)
        .await
        .unwrap();
    let conn = connection.acquire().await?;

    Ok(conn)
}

