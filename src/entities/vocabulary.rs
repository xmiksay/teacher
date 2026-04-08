use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "vocabulary")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub profile_id: Uuid,
    pub word: String,
    pub translation: String,
    pub added_by: String,
    pub context: Option<String>,
    pub last_practiced: DateTimeWithTimeZone,
    pub error_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_language_profile::Entity",
        from = "Column::ProfileId",
        to = "super::user_language_profile::Column::Id"
    )]
    Profile,
}

impl Related<super::user_language_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
