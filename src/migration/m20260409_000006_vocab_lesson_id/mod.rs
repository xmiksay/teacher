use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260409_000006_vocab_lesson_id"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Vocabulary::Table)
                    .add_column(ColumnDef::new(Vocabulary::LessonId).uuid().null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(Vocabulary::Table)
                            .from_col(Vocabulary::LessonId)
                            .to_tbl(Lesson::Table)
                            .to_col(Lesson::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Vocabulary::Table)
                    .drop_column(Vocabulary::LessonId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Vocabulary {
    Table,
    LessonId,
}

#[derive(DeriveIden)]
enum Lesson {
    Table,
    Id,
}
