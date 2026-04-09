use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lesson")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub profile_id: Uuid,
    pub title: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user_language_profile::Entity",
        from = "Column::ProfileId",
        to = "super::user_language_profile::Column::Id"
    )]
    Profile,
    #[sea_orm(has_many = "super::lesson_message::Entity")]
    Messages,
    #[sea_orm(has_many = "super::vocabulary::Entity")]
    Vocabulary,
}

impl Related<super::user_language_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Profile.def()
    }
}

impl Related<super::lesson_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Messages.def()
    }
}

impl Related<super::vocabulary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vocabulary.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
