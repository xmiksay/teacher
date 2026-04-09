use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lesson_message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub lesson_id: Uuid,
    pub role: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::lesson::Entity",
        from = "Column::LessonId",
        to = "super::lesson::Column::Id"
    )]
    Lesson,
}

impl Related<super::lesson::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lesson.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
