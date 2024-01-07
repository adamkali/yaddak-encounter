use sqlx::PgPool;
use uuid::Uuid;
use crate::models::errors::SResult;

pub trait Repo<'a, T>
where T: serde::Serialize + serde::Deserialize<'a> {
    async fn migrate(client: PgPool) -> SResult<()>;
    async fn get(client: PgPool, id: Uuid) -> SResult<T>;
    async fn get_all(client: PgPool ) -> SResult<Vec<T>>;
    async fn post(client: PgPool, model: &T) ->  SResult<()>;
    async fn put(client: PgPool, id: Uuid, model: &T) -> SResult<()>;
    async fn delete(client: PgPool, id: Uuid) -> SResult<()>;
}

