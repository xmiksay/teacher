use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240408_000001_init"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserLanguageProfile::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserLanguageProfile::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(UserLanguageProfile::UserId).string().not_null())
                    .col(ColumnDef::new(UserLanguageProfile::Language).string().not_null())
                    .col(ColumnDef::new(UserLanguageProfile::Level).string().not_null().default("A1"))
                    .col(ColumnDef::new(UserLanguageProfile::Style).string().not_null().default("friendly"))
                    .col(ColumnDef::new(UserLanguageProfile::ExplanationLanguage).string().not_null().default("en"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Vocabulary::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Vocabulary::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Vocabulary::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(Vocabulary::Word).string().not_null())
                    .col(ColumnDef::new(Vocabulary::Translation).string().not_null())
                    .col(ColumnDef::new(Vocabulary::AddedBy).string().not_null().default("user"))
                    .col(ColumnDef::new(Vocabulary::Context).string().null())
                    .col(ColumnDef::new(Vocabulary::LastPracticed).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Vocabulary::ErrorCount).integer().not_null().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Vocabulary::Table, Vocabulary::ProfileId)
                            .to(UserLanguageProfile::Table, UserLanguageProfile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(WeakPoint::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(WeakPoint::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WeakPoint::ProfileId).uuid().not_null())
                    .col(ColumnDef::new(WeakPoint::Type).string().not_null())
                    .col(ColumnDef::new(WeakPoint::Detail).string().not_null())
                    .col(ColumnDef::new(WeakPoint::Active).boolean().not_null().default(true))
                    .foreign_key(
                        ForeignKey::create()
                            .from(WeakPoint::Table, WeakPoint::ProfileId)
                            .to(UserLanguageProfile::Table, UserLanguageProfile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(WeakPoint::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Vocabulary::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UserLanguageProfile::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserLanguageProfile {
    Table,
    Id,
    UserId,
    Language,
    Level,
    Style,
    ExplanationLanguage,
}

#[derive(DeriveIden)]
enum Vocabulary {
    Table,
    Id,
    ProfileId,
    Word,
    Translation,
    AddedBy,
    Context,
    LastPracticed,
    ErrorCount,
}

#[derive(DeriveIden)]
enum WeakPoint {
    Table,
    Id,
    ProfileId,
    Type,
    Detail,
    Active,
}
