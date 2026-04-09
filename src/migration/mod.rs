use sea_orm_migration::prelude::*;

mod m20240408_000001_init;
mod m20260408_000002_auth;
mod m20260408_000003_lessons;
mod m20260408_000004_personal_note;
mod m20260409_000005_lesson_messages;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240408_000001_init::Migration),
            Box::new(m20260408_000002_auth::Migration),
            Box::new(m20260408_000003_lessons::Migration),
            Box::new(m20260408_000004_personal_note::Migration),
            Box::new(m20260409_000005_lesson_messages::Migration),
        ]
    }
}
