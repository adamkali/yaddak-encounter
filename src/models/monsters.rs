use std::path::PathBuf;

use sea_query::{Iden, Table, ColumnDef, PostgresQueryBuilder, Query, Expr, ForeignKey, Value};
use sea_query_binder::SqlxBinder;
use serde::{Serialize, Deserialize};
use sqlx::{FromRow, query, Statement, query_as_with, query_with};
use tokio::{fs::File, io::AsyncReadExt};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::traits::repo::{Repo, connect};

use super::errors::SResult;

pub static WOTCUUID: &str = "ba4726f1-5df7-4798-a530-66ad845e1b05";

#[derive(Serialize, Deserialize, Debug,
         Clone, Default, FromRow,
         ToSchema)]
pub struct Monster {
    pub id: Uuid,
    pub name: String,
    pub meta: String,
    pub armor_class: String,
    pub hit_points: String,
    pub speed: String,
    pub str: i16,
    pub dex: i16,
    pub con: i16,
    pub int: i16,
    pub wis: i16,
    pub cha: i16,
    pub saving_throws: String,
    pub skills: String,
    pub senses: String,
    pub languages: String,
    pub challenge: f32,
    pub traits: Option<String>,
    pub actions: String,
    pub damage_immunities: Option<String>,
    pub condition_immunities: Option<String>,
    pub legendary_actions: Option<String>,
    pub img_url: String,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug,
         Clone, Default)]
pub struct MonsterFromJson {
    pub name: String,
    pub meta: String,
    pub armor_class: String, 
    pub hit_points: String,
    pub speed: String,
    pub str: i16,
    pub dex: i16,
    pub con: i16,
    pub int: i16,
    pub wis: i16,
    pub cha: i16,
    pub saving_throws: String,
    pub skills: String,
    pub senses: String,
    pub languages: String,
    pub challenge: String,
    pub traits: Option<String>,
    pub actions: String,
    pub damage_immunities: Option<String>,
    pub condition_immunities: Option<String>,
    pub legendary_actions: Option<String>,
    pub img_url: String
}

impl Monster {
    pub async fn migrate_json(con_str: String) -> SResult<()> {
        let file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("monsters.json");

        let mut file_content = String::new();
        let mut file = File::open(file_path).await?;
        file.read_to_string(&mut file_content).await?;

        let monsters_to_convert: Vec<MonsterFromJson> = serde_json::from_str(&file_content)?;
        let monsters_to_post: Vec<Monster> = monsters_to_convert
            .iter()
            .map(|m| (*m).clone().into())
            .collect();

        let mut client = connect(con_str.clone()).await?;
        let (sql, values) = Query::select()
            .from(MonsterModel::Table)
            .columns(MonsterModel::cols()) 
            .and_where(Expr::col(MonsterModel::Name).eq(String::from("Aboleth")))
            .build_sqlx(PostgresQueryBuilder);

        let rows: Vec<Monster> = query_as_with(&sql, values.clone())
            .fetch_all(&mut *client)
            .await?;

        if rows.is_empty() {
            for ele in monsters_to_post {
                Self::post(con_str.clone(), &ele.clone()).await?;
            }
        }

        Ok(())
    }
}

impl From<MonsterFromJson> for Monster {
    fn from(value: MonsterFromJson) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: value.name,
            meta: value.meta,
            armor_class: value.armor_class,
            hit_points: value.hit_points,
            speed: value.speed,
            str: value.str,
            dex: value.dex,
            con: value.con,
            int: value.int,
            wis: value.wis,
            cha: value.cha,
            saving_throws: value.saving_throws,
            skills: value.skills,
            senses: value.senses,
            languages: value.languages,
            challenge: challenge_convert(value
                                            .challenge
                                            .rsplit(' ')
                                            .next_back()
                                            .unwrap_or("")
                                        ),
            traits: value.traits,
            actions: value.actions,
            damage_immunities: value.damage_immunities,
            condition_immunities: value.condition_immunities,
            legendary_actions: value.legendary_actions,
            img_url: value.img_url,
            user_id: Uuid::parse_str(WOTCUUID).unwrap()
        }
    }
}

fn challenge_convert(string: &str) -> f32 {
    match string {
        "1/8" => 0.125,
        "1/4" => 0.25,
        "1/2" => 0.5,
        _ => {
            match string.parse::<f32>() {
                Ok(a) => a,
                Err(e) => 0.0,
            }
        }
    }
}

#[derive(Iden)]
enum MonsterModel {
    Table,
    Id,
    Name,
    Meta,
    ArmorClass,
    HitPoints,
    Speed,
    Str,
    Dex,
    Con,
    Int,
    Wis,
    Cha,
    SavingThrows,
    Skills,
    Senses,
    Languages,
    Challenge,
    Traits,
    Actions,
    DamageImmunities,
    ConditionImmunities,
    LegendaryActions,
    ImgUrl,
    UserId,
}

impl MonsterModel {
    pub fn cols() -> Vec<Self> {
        return vec![
            Self::Id,
            Self::Name,
            Self::Meta,
            Self::ArmorClass,
            Self::HitPoints,
            Self::Speed,
            Self::Str,
            Self::Dex,
            Self::Con,
            Self::Int,
            Self::Wis,
            Self::Cha,
            Self::SavingThrows,
            Self::Skills,
            Self::Senses,
            Self::Languages,
            Self::Challenge,
            Self::Traits,
            Self::Actions,
            Self::DamageImmunities,
            Self::ConditionImmunities,
            Self::LegendaryActions,
            Self::ImgUrl,
            Self::UserId
        ]
    }
}

impl Repo<'_, Monster> for Monster {
    async fn migrate(con_str: String) -> super::errors::SResult<()> {
        let mut client = connect(con_str).await?;


        let sql = Table::create()
            .table(MonsterModel::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(MonsterModel::Id)
                    .uuid()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(MonsterModel::Name).string().not_null())
            .col(ColumnDef::new(MonsterModel::Meta).string().not_null())
            .col(ColumnDef::new(MonsterModel::ArmorClass).string().not_null())
            .col(ColumnDef::new(MonsterModel::HitPoints).string().not_null())
            .col(ColumnDef::new(MonsterModel::Speed).string().not_null())
            .col(ColumnDef::new(MonsterModel::Str).integer().not_null())
            .col(ColumnDef::new(MonsterModel::Dex).integer().not_null())
            .col(ColumnDef::new(MonsterModel::Con).integer().not_null())
            .col(ColumnDef::new(MonsterModel::Int).integer().not_null())
            .col(ColumnDef::new(MonsterModel::Wis).integer().not_null())
            .col(ColumnDef::new(MonsterModel::Cha).integer().not_null())
            .col(ColumnDef::new(MonsterModel::SavingThrows).string().not_null())
            .col(ColumnDef::new(MonsterModel::Skills).string().not_null())
            .col(ColumnDef::new(MonsterModel::Senses).string().not_null())
            .col(ColumnDef::new(MonsterModel::Languages).string().not_null())
            .col(ColumnDef::new(MonsterModel::Challenge).float().not_null())
            .col(ColumnDef::new(MonsterModel::Actions).string().not_null())
            .col(ColumnDef::new(MonsterModel::Traits).string().default(Value::String(None)))
            .col(ColumnDef::new(MonsterModel::LegendaryActions).string().default(Value::String(None)))
            .col(ColumnDef::new(MonsterModel::ConditionImmunities).string().default(Value::String(None)))
            .col(ColumnDef::new(MonsterModel::DamageImmunities).string().default(Value::String(None)))
            .col(ColumnDef::new(MonsterModel::ImgUrl).string().not_null())
            .col(ColumnDef::new(MonsterModel::UserId).uuid().not_null())
            .foreign_key(ForeignKey::create()
                           .name("FK_User")
                           .from(MonsterModel::Table, MonsterModel::UserId)
                           .to(super::user::UserModel::Table,
                               super::user::UserModel::Id)
                           .on_delete(sea_query::ForeignKeyAction::Cascade)
                           .on_update(sea_query::ForeignKeyAction::Cascade)
                        )
            .build(PostgresQueryBuilder);

        let _ = query(sql.as_str())
            .execute(&mut *client)
            .await;

        Ok(())
    }

    async fn get(con_str: String, id: Uuid) -> super::errors::SResult<Monster> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::select()
            .columns(MonsterModel::cols()) 
            .from(MonsterModel::Table)
            .limit(1)
            .and_where(Expr::col(MonsterModel::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let rows: Monster = query_as_with(&sql, values.clone())
            .fetch_one(&mut *client)
            .await?;

        Ok(rows.into())
    }

    async fn get_all(con_str: String ) -> super::errors::SResult<Vec<Monster>> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::select()
            .columns(MonsterModel::cols()) 
            .from(MonsterModel::Table)
            .build_sqlx(PostgresQueryBuilder);

        let rows: Vec<Monster> = query_as_with(&sql, values.clone())
            .fetch_all(&mut *client)
            .await?;

        Ok(rows.into())
    }

    async fn post(con_str: String, model: &Monster) ->  super::errors::SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::insert()
            .into_table(MonsterModel::Table)
            .columns(MonsterModel::cols())
            .values_panic([
                model.id.into(),
                model.name.clone().into(),
                model.meta.clone().into(),
                model.armor_class.clone().into(),
                model.hit_points.clone().into(),
                model.speed.clone().into(),
                model.str.to_string().into(),
                model.dex.to_string().into(),
                model.con.to_string().into(),
                model.int.to_string().into(),
                model.wis.to_string().into(),
                model.cha.to_string().into(),
                model.saving_throws.clone().into(),
                model.skills.clone().into(),
                model.senses.clone().into(),
                model.languages.clone().into(),
                model.challenge.to_string().into(),
                model.traits.clone().into(),
                model.actions.clone().into(),
                model.damage_immunities.clone().into(),
                model.condition_immunities.clone().into(),
                model.legendary_actions.clone().into(),
                model.img_url.clone().into()
            ])
            .build_sqlx(PostgresQueryBuilder);
        let _ = query_with(&sql, values)
            .execute(&mut *client)
            .await?;

        Ok(())
    }

    async fn put(con_str: String, id: Uuid, model: &Monster) -> super::errors::SResult<()> {
        todo!()
    }

    async fn delete(con_str: String, id: Uuid) -> super::errors::SResult<()> {
        let mut client = connect(con_str).await?;
        let (sql, values) = Query::update()
            .table(MonsterModel::Table)
            .and_where(Expr::col(MonsterModel::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);
        
        let _ = query_with(sql.as_str(), values)
            .execute(&mut *client)
            .await?;
        Ok(())
    }
}
