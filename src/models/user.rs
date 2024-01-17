use std::env;

use sqlx::{
    query,
    query_as, FromRow, query_as_with, query_with
};
use sea_query::{
    Iden,
    Query,
    PostgresQueryBuilder,
    Expr,
    Table,
    ColumnDef
};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::ToSchema;
use uuid::{Uuid, fmt::Hyphenated};
use argon2::{self, Config, Variant, Version};

use crate::traits::repo::{Repo, connect};

use super::errors::{YaddakError, SResult};

#[derive(Serialize, Deserialize, Debug,
         Clone, Default, FromRow,
         ToSchema)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub user_auth: String,
}

#[derive(Deserialize, Debug, Clone,
         ToSchema)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub user_email: String,
    pub user_pass: String,
}

#[derive(Deserialize, Debug, Clone,
         ToSchema)]
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
        con_str: String,
        user_name: String,
        user_email: String,
        user_pass: String
    ) -> SResult<Self> {
        debug!("Checking username");
        User::user_name_not_used(con_str.clone(), user_name.clone()).await?;
        debug!("Checking email");
        User::user_email_not_used(con_str.clone(), user_email.clone()).await?;
        let mut user: User = User {
            user_email,
            user_name,
            id: Uuid::new_v4(),
            user_auth: String::new(),
        };
        user.hash_password(user_pass)?;
        
        User::post(con_str, &user.clone()).await?;
        
        Ok(user)
    }

    fn hash_password(&mut self, user_pass: String) -> SResult<()> {
        // Retrieve salt from environment variabl
        let salt: String = env::var("YADDAK_SECRET")?; 

        let config = new_config(&salt.as_bytes());

        // Concatenate data for hashing
        let concat_data = format!("{}:{}", self.user_name, user_pass);

        // Hash the password
        self.user_auth = argon2::hash_encoded(
            concat_data.as_bytes(),
            self.id.as_bytes(),
            &config)?
            .rsplit('$')
            .next()
            .unwrap()
            .to_string();

        
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

        let salt: String = env::var("YADDAK_SECRET")?; 
        let config = new_config(salt.as_bytes());
       
        let generated_auth = argon2::hash_encoded(
            concat_data.as_bytes(),
            self.id.as_bytes(),
            &config)?
            .rsplit('$')
            .next()
            .unwrap()
            .to_string();

        if generated_auth.ne(&self.user_auth) {
            return Err(YaddakError::authorize_error("Could not Sign In".to_owned()));
        }
        
        Ok(())
    }

    pub async fn user_email_not_used(con_str: String, user_email: String) -> SResult<()> {
        let mut client = connect(con_str).await?;

        let (sql, values) = Query::select()
            .from(UserModel::Table)
            .limit(1)
            .and_where(Expr::col(UserModel::UserEmail).eq(user_email))
            .build_sqlx(PostgresQueryBuilder);


        let rows: Vec<User> = query_as_with(&sql, values)
            .fetch_all(&mut *client)
            .await?;

        if !rows.is_empty() {
            return Err(YaddakError::authorize_error("That email is already used".to_string()));
        }

        Ok(())
    }

    pub async fn user_name_not_used(con_str: String, user_name: String) -> SResult<()> {
        let rows = get_users_by_username(
            con_str,
            user_name
        ).await?;
        if !rows.is_empty() {
            return Err(YaddakError::authorize_error("Username is already used".to_string()));
        }

        Ok(())
    }

    pub async fn get_user_by_name(
        con_str: String,
        user_name: String
    ) -> SResult<User> {
        let rows = get_users_by_username(
            con_str,
            user_name.clone()
        ).await?;
        debug!("{:?}",rows);
        debug!("{:?}",user_name);
        if rows.len() != 1 {
            return Err(YaddakError::authorize_error("Not Found".to_string()));
        } else {
            Ok(User::from(rows[0].clone()))
        }
    }

    pub async fn check_auth(
        con_str: String,
        auth_header:String
    ) -> SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::select()
            .from(UserModel::Table)
            .columns([
                UserModel::UserAuth,
            ]) 
            .and_where(Expr::col(UserModel::UserAuth).eq(auth_header))
            .build_sqlx(PostgresQueryBuilder);

        let rows: Vec<_> = query_with(&sql, values.clone())
            .fetch_all(&mut *client)
            .await?;

        if rows.is_empty() {
            return Err(YaddakError::authorize_error("Could not authenticate".to_string()));
        }
        Ok(())
    }
}

async fn get_users_by_username(
    con_str: String,
    user_name: String
) -> SResult<Vec<User>> {
    let mut client = connect(con_str).await?;
    let (sql, values) = Query::select()
        .from(UserModel::Table)
        .columns([
            UserModel::Id,
            UserModel::UserEmail,
            UserModel::UserName,
            UserModel::UserAuth,
        ]) 
        .and_where(Expr::col(UserModel::UserName).eq(user_name))
        .build_sqlx(PostgresQueryBuilder);

    let rows: Vec<User> = query_as_with(&sql, values.clone())
        .fetch_all(&mut *client)
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
pub(crate) enum UserModel {
    Table,
    Id,
    UserName,
    UserAuth,
    UserEmail
}

impl Repo<'_, User> for User {
    async fn get(con_str: String, id: Uuid) -> SResult<User> {
        let mut client = connect(con_str).await?;

        let (sql, values) = Query::select()
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ]) 
            .from(UserModel::Table)
            .limit(1)
            .and_where(Expr::col(UserModel::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        debug!("query:\t{}", sql.clone());
        debug!("values:\t{:?}", values.clone());
        let rows: User = query_as_with(&sql, values.clone())
            .fetch_one(&mut *client)
            .await?;

        Ok(rows.into())
            
    }

    async fn get_all(con_str: String ) -> SResult<Vec<User>> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::select()
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ]) 
            .from(UserModel::Table)
            .build_sqlx(PostgresQueryBuilder);

        let users: Vec<User> = query_as_with(&sql, values)
            .fetch_all(&mut *client)
            .await?;

        Ok(users)
    }

    async fn post(con_str: String, model: &User) ->  SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::insert()
            .into_table(UserModel::Table)
            .columns([
                UserModel::Id,
                UserModel::UserEmail,
                UserModel::UserName,
                UserModel::UserAuth,
            ])
            .values_panic([
                model.id.into(),
                model.user_email.clone().into(),
                model.user_name.clone().into(),
                model.user_auth.clone().into(),
            ])
            .build_sqlx(PostgresQueryBuilder);
        let _ = query_with(&sql, values)
            .execute(&mut *client)
            .await?;

        Ok(())
    }

    async fn put(con_str: String, id: Uuid, model: &User) -> SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::update()
            .table(UserModel::Table)
            .values([
                (UserModel::UserName, model.user_name.clone().into()),
                (UserModel::UserEmail, model.user_email.clone().into()),
                (UserModel::UserAuth, model.user_auth.clone().into()),

            ])
            .and_where(Expr::col(UserModel::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);
        
        let _ = query_with(sql.as_str(), values)
            .execute(&mut *client)
            .await?;
        
        Ok(())
        
    }

    async fn delete(con_str: String, id: Uuid) -> SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::update()
            .table(UserModel::Table)
            .and_where(Expr::col(UserModel::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);
        
        let _ = query_with(sql.as_str(), values)
            .execute(&mut *client)
            .await?;
        Ok(())
    }

    async fn migrate(con_str: String ) -> SResult<()> {
        let mut client = connect(con_str).await?;
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
            .execute(&mut *client)
            .await?;
        Ok(())
    }
}
