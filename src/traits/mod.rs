pub mod repo;

use sqlx::PgPool;

use crate::models::{user::User, errors::SResult};

use self::repo::Repo;

pub async fn migrate(client: PgPool) -> SResult<()> {
    User::migrate(client).await?;
    Ok(())
}
