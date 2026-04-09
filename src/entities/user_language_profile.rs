use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_language_profile")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub language: String,
    pub level: String,
    pub style: String,
    pub explanation_language: String,
    #[sea_orm(column_type = "Text")]
    pub personal_note: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::vocabulary::Entity")]
    Vocabulary,
    #[sea_orm(has_many = "super::weak_point::Entity")]
    WeakPoint,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::vocabulary::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Vocabulary.def()
    }
}

impl Related<super::weak_point::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::WeakPoint.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
