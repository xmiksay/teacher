use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260409_000005_lesson_messages"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create lesson_message table
        manager
            .create_table(
                Table::create()
                    .table(LessonMessage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LessonMessage::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LessonMessage::LessonId).uuid().not_null())
                    .col(ColumnDef::new(LessonMessage::Role).string().not_null())
                    .col(ColumnDef::new(LessonMessage::Content).text().not_null())
                    .col(
                        ColumnDef::new(LessonMessage::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(LessonMessage::Table, LessonMessage::LessonId)
                            .to(Lesson::Table, Lesson::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Migrate existing JSONB data into lesson_message rows
        let db = manager.get_connection();
        let lessons: Vec<(String, String, String)> = db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT id::text, messages::text, created_at::text FROM lesson WHERE messages IS NOT NULL".to_string(),
            ))
            .await?
            .iter()
            .map(|row| {
                let id: String = row.try_get("", "id").unwrap_or_default();
                let messages: String = row.try_get("", "messages").unwrap_or_default();
                let created_at: String = row.try_get("", "created_at").unwrap_or_default();
                (id, messages, created_at)
            })
            .collect();

        for (lesson_id, messages_json, created_at_str) in &lessons {
            let messages: Vec<serde_json::Value> =
                serde_json::from_str(messages_json).unwrap_or_default();

            for (i, msg) in messages.iter().enumerate() {
                let role = msg["role"].as_str().unwrap_or("user");
                let content = msg["content"].as_str().unwrap_or("");
                let uuid = uuid::Uuid::new_v4();

                // Offset each message by i seconds from lesson created_at for ordering
                let content_escaped = content.replace('\'', "''");
                db.execute(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        "INSERT INTO lesson_message (id, lesson_id, role, content, created_at) VALUES ('{uuid}', '{lesson_id}', '{role}', '{content_escaped}', '{created_at_str}'::timestamptz + interval '{i} seconds')",
                    ),
                ))
                .await
                .map_err(|e| DbErr::Custom(format!("Failed to migrate message: {e}")))?;
            }
        }

        // 3. Drop messages column from lesson table
        manager
            .alter_table(
                Table::alter()
                    .table(Lesson::Table)
                    .drop_column(Lesson::Messages)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add messages column back
        manager
            .alter_table(
                Table::alter()
                    .table(Lesson::Table)
                    .add_column(
                        ColumnDef::new(Lesson::Messages)
                            .json_binary()
                            .not_null()
                            .default(sea_orm::sea_query::Expr::cust("'[]'::jsonb")),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Aggregate lesson_message rows back into JSONB
        let db = manager.get_connection();
        db.execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE lesson SET messages = COALESCE(
                (SELECT jsonb_agg(jsonb_build_object('role', lm.role, 'content', lm.content) ORDER BY lm.created_at)
                 FROM lesson_message lm WHERE lm.lesson_id = lesson.id),
                '[]'::jsonb
            )
            "#.to_string(),
        ))
        .await?;

        // 3. Drop lesson_message table
        manager
            .drop_table(Table::drop().table(LessonMessage::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LessonMessage {
    Table,
    Id,
    LessonId,
    Role,
    Content,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Lesson {
    Table,
    Messages,
    Id,
}
