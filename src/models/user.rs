use std::env;

use sea_query_binder::SqlxBinder;
use sqlx::{
    PgPool,
    query,
    query_as_with,
    prelude::FromRow, Postgres, query_as, Statement
};
use sea_query::{
    Iden,
    Query,
    PostgresQueryBuilder,
    Expr,
    Table,
    ColumnDef
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use argon2::{self, Config, Variant, Version};

use crate::traits::repo::Repo;

use super::errors::{YaddakError, SResult};

#[derive(Serialize, Deserialize, Debug,
         Clone, Default, FromRow)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub user_auth: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_pass: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoginUserRequest {
    pub user_name: String,
    pub user_pass: String,
}

impl User {
    pub fn new() -> Self {
        Self {
            id: Uuid::nil(),
            user_name: String::new(),
            user_email: String::new(),
            user_auth: String::new(),
        }
    }

    pub async fn create(
        client: PgPool,
        user_name: String,
        user_email: String,
        user_pass: String
    ) -> SResult<Self> {
        User::user_name_not_used(client.clone(), user_name.clone()).await?;
        User::user_email_not_used(client.clone(), user_email.clone()).await?;
        let mut user: User = User {
            user_email,
            user_name,
            id: Uuid::new_v4(),
            user_auth: String::new(),
        };
        user.hash_password(user_pass)?;
        
        User::post(client, &user.clone()).await?;
        
        Ok(user)
    }

    fn hash_password(&mut self, user_pass: String) -> SResult<()> {
        // Retrieve salt from environment variabl
        let salt: String = env::var("UB_CARD_SECRET")?; 

        let config = new_config(&salt.as_bytes());

        // Concatenate data for hashing
        let concat_data = format!("{}:{}", self.user_name, user_pass);

        // Hash the password
        self.user_auth = argon2::hash_encoded(
            concat_data.as_bytes(),
            self.id.as_bytes(),
            &config)?;
        
        Ok(())

    }

    pub fn authenticate(
        &self,
        user_name: String,
        user_pass: String
    ) -> SResult<()> {

        let concat_data = format!(
            "{}:{}",
            user_name,
            user_pass,
        );

        let salt = env::var("UB_CARD_SECRET")?;
        let config = new_config(salt.as_bytes());
       
        let generated_auth = argon2::hash_encoded(
            concat_data.as_bytes(),
            self.id.as_bytes(),
            &config)?;

        if generated_auth.ne(&self.user_auth) {
            return Err(YaddakError::authorize_error("Could not Sign In".to_owned()));
        }
        
        Ok(())
    }

    pub async fn user_email_not_used(client: PgPool, user_email: String) -> SResult<()> {

        let (sql, _) = Query::select()
            .from(UserModel::Table)
            .limit(1)
            .and_where(Expr::col(UserModel::UserEmail).like(user_email))
            .build(PostgresQueryBuilder);


        let rows = query(&sql)
            .fetch_all(&client)
            .await?;

        if !rows.is_empty() {
            return Err(YaddakError::authorize_error("That email is already used".to_string()));
        }

        Ok(())
    }

    pub async fn user_name_not_used(client: PgPool, user_name: String) -> SResult<()> {
        let rows = get_users_by_username(
            client,
            user_name
        ).await?;
        if !rows.is_empty() {
            return Err(YaddakError::authorize_error("Username is already used".to_string()));
        }

        Ok(())
    }

    pub async fn get_user_by_name(
        client: PgPool,
        user_name: String
    ) -> SResult<User> {
        let rows = get_users_by_username(
            client,
            user_name
        ).await?;
        if rows.len() != 1 {
            return Err(YaddakError::authorize_error("Not Found".to_string()));
        } else {
            Ok(User::from(rows[0].clone()))
        }
    }
}

async fn get_users_by_username(
    client: PgPool,
    user_name: String
) -> SResult<Vec<User>> {
    let (sql, _values) = Query::select()
        .from(UserModel::Table)
        .and_where(Expr::col(UserModel::UserEmail).like(user_name))
        .build(PostgresQueryBuilder);

    let rows: Vec<User> = query_as(&sql)
        .fetch_all(&client)
        .await?;

    Ok(rows)
}

fn new_config<'a>(salt: &[u8])
-> Config {
    Config {
        hash_length: 32,
        ad: &[],
        lanes: 4,
        mem_cost: 65536,
        secret: salt,
        time_cost: 10,
        variant: Variant::Argon2i,
        version: Version::Version13
    }
}


#[derive(Iden)]
enum UserModel {
    Table,
    Id,
    UserName,
    UserAuth,
    UserEmail
}

impl Repo<'_, User> for User {
    async fn get(client: PgPool, id: Uuid) -> SResult<User> {
        let (sql, _values) = Query::select()
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ]) 
            .from(UserModel::Table)
            .limit(1)
            .and_where(Expr::col(UserModel::Id).like(id))
            .build(PostgresQueryBuilder);

        let rows: User = query_as(&sql)
            .fetch_one(&client)
            .await?;

        Ok(rows.into())
            
    }

    async fn get_all(client: PgPool ) -> SResult<Vec<User>> {
        let (sql, _values) = Query::select()
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ]) 
            .from(UserModel::Table)
            .build(PostgresQueryBuilder);

        let users: Vec<User> = query_as(&sql)
            .fetch_all(&client)
            .await?;

        Ok(users)
    }

    async fn post(client: PgPool, model: &User) ->  SResult<()> {
        let (sql, _values) = Query::insert()
            .into_table(UserModel::Table)
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ])
            .values_panic([
                model.id.to_string().into(),
                model.user_email.clone().into(),
                model.user_name.clone().into(),
                model.user_auth.clone().into(),
            ])
            .build(PostgresQueryBuilder);
        let _ = query(&sql)
            .execute(&client)
            .await?;

        Ok(())
    }

    async fn put(client: PgPool, id: Uuid, model: &User) -> SResult<()> {
        let (sql, _values) = Query::update()
            .table(UserModel::Table)
            .values([
                (UserModel::UserName, model.user_name.clone().into()),
                (UserModel::UserEmail, model.user_email.clone().into()),
                (UserModel::UserAuth, model.user_auth.clone().into()),

            ])
            .and_where(Expr::col(UserModel::Id).like(id))
            .build(PostgresQueryBuilder);
        
        let _ = query(sql.as_str())
            .execute(&client)
            .await?;
        
        Ok(())
        
    }

    async fn delete(client: PgPool, id: Uuid) -> SResult<()> {
        let (sql, _values) = Query::update()
            .table(UserModel::Table)
            .and_where(Expr::col(UserModel::Id).like(id))
            .build(PostgresQueryBuilder);
        
        let _ = query(sql.as_str())
            .execute(&client)
            .await?;
        Ok(())
    }

    async fn migrate(client: PgPool ) -> SResult<()> {
        let sql = Table::create()
            .table(UserModel::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UserModel::Id)
                    .uuid()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(UserModel::UserAuth).string().not_null())
            .col(ColumnDef::new(UserModel::UserName).string().not_null())
            .col(ColumnDef::new(UserModel::UserEmail).string().not_null())
            .build(PostgresQueryBuilder);

        let _ = query(sql.as_str())
            .execute(&client)
            .await?;
        Ok(())
    }
}
