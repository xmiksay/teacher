use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260408_000003_lessons"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Lesson::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Lesson::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Lesson::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(Lesson::Title).string().not_null())
                    .col(ColumnDef::new(Lesson::Messages).json_binary().not_null())
                    .col(
                        ColumnDef::new(Lesson::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Lesson::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Lesson::Table, Lesson::ProfileId)
                            .to(UserLanguageProfile::Table, UserLanguageProfile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Lesson::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Lesson {
    Table,
    Id,
    ProfileId,
    Title,
    Messages,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserLanguageProfile {
    Table,
    Id,
}
