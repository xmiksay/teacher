use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260408_000004_personal_note"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .add_column(
                        ColumnDef::new(UserLanguageProfile::PersonalNote)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .drop_column(UserLanguageProfile::PersonalNote)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum UserLanguageProfile {
    Table,
    PersonalNote,
}
