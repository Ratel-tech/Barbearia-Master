mod app;
mod auth_protection;
mod db;
mod error;
mod models;

use anyhow::Context;
use app::router;
use db::{Db, migrate, seed};
use std::{env, fs, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env_file(".env");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stitch_barbershop_api=debug,tower_http=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://database.db?mode=rwc".into());
    let db = Db::connect(&database_url).await?;
    migrate(&db).await?;
    seed(&db).await?;

    let app = router(db)
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = env::var("API_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into())
        .parse()
        .context("invalid API_ADDR")?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "stitch barbershop API listening");

    axum::serve(listener, app).await?;
    Ok(())
}

fn load_env_file(path: &str) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    for line in contents.lines() {
        let Some((key, value)) = parse_env_line(line) else {
            continue;
        };
        if env::var_os(&key).is_none() {
            unsafe {
                env::set_var(key, value);
            }
        }
    }
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_string(), value.trim().trim_matches('"').to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_simple_env_assignment() {
        assert_eq!(
            super::parse_env_line("SEED_DEMO_DATA=1"),
            Some(("SEED_DEMO_DATA".to_string(), "1".to_string()))
        );
    }

    #[test]
    fn ignores_blank_comments_and_invalid_env_lines() {
        assert_eq!(super::parse_env_line(""), None);
        assert_eq!(super::parse_env_line("# comment"), None);
        assert_eq!(super::parse_env_line("not-an-assignment"), None);
    }
}
