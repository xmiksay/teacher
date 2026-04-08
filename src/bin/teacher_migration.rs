use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tracing_subscriber::EnvFilter;

use teacher_server::migration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://teacher:teacher@localhost:5432/teacher".to_string());

    let db = Database::connect(&database_url).await?;
    tracing::info!("Connected to database");

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("up") => {
            migration::Migrator::up(&db, None).await?;
            tracing::info!("Migrations applied");
        }
        Some("down") => {
            let steps = args.get(2).and_then(|s| s.parse().ok());
            migration::Migrator::down(&db, steps).await?;
            tracing::info!("Migrations rolled back");
        }
        Some("status") => {
            migration::Migrator::status(&db).await?;
        }
        _ => {
            eprintln!("Usage: teacher_migration <up|down|status> [steps]");
            std::process::exit(1);
        }
    }

    Ok(())
}
