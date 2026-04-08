use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260408_000002_auth"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create user table (with password_hash inline)
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create auth_token table
        manager
            .create_table(
                Table::create()
                    .table(AuthToken::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AuthToken::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AuthToken::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuthToken::Token).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(AuthToken::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AuthToken::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AuthToken::Table, AuthToken::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Alter user_language_profile: change user_id from string to uuid FK
        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .drop_column(UserLanguageProfile::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .add_column(ColumnDef::new(UserLanguageProfile::UserId).uuid().not_null())
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .from_tbl(UserLanguageProfile::Table)
                            .from_col(UserLanguageProfile::UserId)
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .to_owned(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: restore user_id as string, drop auth tables
        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .drop_column(UserLanguageProfile::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserLanguageProfile::Table)
                    .add_column(
                        ColumnDef::new(UserLanguageProfile::UserId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AuthToken::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AuthToken {
    Table,
    Id,
    UserId,
    Token,
    ExpiresAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserLanguageProfile {
    Table,
    UserId,
}
