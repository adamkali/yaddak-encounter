use sqlx::PgPool;

pub struct YaddakState { pub db: PgPool, }
