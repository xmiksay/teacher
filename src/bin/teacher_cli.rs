use sea_orm::Database;
use tracing_subscriber::EnvFilter;

use teacher_server::entities::{user_language_profile, vocabulary, weak_point};

use sea_orm::{EntityTrait, QueryFilter, QueryOrder, QuerySelect, ColumnTrait};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://teacher:teacher@localhost:5432/teacher".to_string());

    let db = Database::connect(&database_url).await?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("profiles") => {
            let profiles = user_language_profile::Entity::find().all(&db).await?;
            for p in profiles {
                println!("{}\t{}\t{}\t{}", p.id, p.language, p.level, p.style);
            }
        }
        Some("vocab") => {
            let profile_id: Uuid = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Usage: teacher_cli vocab <profile_id>"))?
                .parse()?;

            let words = vocabulary::Entity::find()
                .filter(vocabulary::Column::ProfileId.eq(profile_id))
                .order_by_asc(vocabulary::Column::LastPracticed)
                .limit(50)
                .all(&db)
                .await?;

            for w in words {
                println!(
                    "{}\t{}\t{}\t{}\terrors:{}",
                    w.id, w.word, w.translation, w.added_by, w.error_count
                );
            }
        }
        Some("weak-points") => {
            let profile_id: Uuid = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Usage: teacher_cli weak-points <profile_id>"))?
                .parse()?;

            let points = weak_point::Entity::find()
                .filter(weak_point::Column::ProfileId.eq(profile_id))
                .filter(weak_point::Column::Active.eq(true))
                .all(&db)
                .await?;

            for wp in points {
                println!("{}\t[{}]\t{}", wp.id, wp.r#type, wp.detail);
            }
        }
        _ => {
            eprintln!("Usage: teacher_cli <profiles|vocab|weak-points> [args...]");
            std::process::exit(1);
        }
    }

    Ok(())
}
