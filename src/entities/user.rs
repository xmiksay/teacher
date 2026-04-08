use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::auth_token::Entity")]
    AuthToken,
    #[sea_orm(has_many = "super::user_language_profile::Entity")]
    UserLanguageProfile,
}

impl Related<super::auth_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuthToken.def()
    }
}

impl Related<super::user_language_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserLanguageProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
