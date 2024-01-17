pub mod repo;

use sqlx::PgPool;

use crate::models::{user::User, errors::SResult, monsters::Monster};

use self::repo::Repo;

pub async fn migrate(client: String) -> SResult<()> {
    User::migrate(client.clone()).await?;
    Monster::migrate(client.clone()).await?;
    Monster::migrate_json(client.clone()).await?;
    Ok(())
}
