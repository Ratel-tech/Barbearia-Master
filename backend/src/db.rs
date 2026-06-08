use anyhow::Context;
use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, rand_core::OsRng};
use sqlx::{Executor, SqlitePool, sqlite::SqlitePoolOptions};
use std::{fs, path::Path};

#[derive(Clone)]
pub struct Db {
    pub pool: SqlitePool,
}

impl Db {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        if let Some(raw_path) = database_url.strip_prefix("sqlite://") {
            let path = raw_path.split('?').next().unwrap_or(raw_path);
            if path != ":memory:"
                && let Some(parent) = Path::new(path).parent()
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create database directory {parent:?}"))?;
            }
        }
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect(database_url)
            .await
            .context("failed to connect database")?;
        Ok(Self { pool })
    }
}

pub async fn migrate(db: &Db) -> anyhow::Result<()> {
    let schema = include_str!("../migrations/001_init.sql");
    for statement in schema_statements(schema, false) {
        db.pool.execute(statement.as_str()).await?;
    }
    ensure_column(
        &db.pool,
        "users",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "clients",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "barbers",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(&db.pool, "barbers", "document", "text not null default ''").await?;
    ensure_column(&db.pool, "barbers", "email", "text not null default ''").await?;
    ensure_column(
        &db.pool,
        "barbers",
        "password_hash",
        "text not null default ''",
    )
    .await?;
    ensure_column(
        &db.pool,
        "services",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "appointments",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "payments",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "extra_expenses",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    ensure_column(
        &db.pool,
        "audit_logs",
        "barbershop_id",
        "integer references barbershops(id)",
    )
    .await?;
    for statement in schema_statements(schema, true) {
        db.pool.execute(statement.as_str()).await?;
    }
    Ok(())
}

fn schema_statements(schema: &str, indexes: bool) -> Vec<String> {
    schema
        .split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .filter(|statement| is_index_statement(statement) == indexes)
        .map(|statement| format!("{statement};"))
        .collect()
}

fn is_index_statement(statement: &str) -> bool {
    let statement = statement.trim_start().to_ascii_lowercase();
    statement.starts_with("create index") || statement.starts_with("create unique index")
}

async fn ensure_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    definition: &str,
) -> anyhow::Result<()> {
    let pragma = format!("pragma table_info({table})");
    let columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
        sqlx::query_as(&pragma).fetch_all(pool).await?;
    if columns.iter().any(|(_, name, _, _, _, _)| name == column) {
        return Ok(());
    }
    let sql = format!("alter table {table} add column {column} {definition}");
    pool.execute(sql.as_str()).await?;
    Ok(())
}

pub async fn seed(db: &Db) -> anyhow::Result<()> {
    if std::env::var("SEED_DEMO_DATA").ok().as_deref() != Some("1") {
        return Ok(());
    }
    seed_demo_data(db).await
}

async fn seed_demo_data(db: &Db) -> anyhow::Result<()> {
    let admin_email =
        std::env::var("DEMO_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.test".to_string());
    let admin_password = demo_admin_password()?;

    let mut tx = db.pool.begin().await?;
    sqlx::query("insert or ignore into barbershops (name, slug) values (?, ?)")
        .bind("Barbearia Mestre Teste")
        .bind("barbearia-mestre-teste")
        .execute(&mut *tx)
        .await?;
    let barbershop_id: (i64,) =
        sqlx::query_as("select id from barbershops where slug = 'barbearia-mestre-teste'")
            .fetch_one(&mut *tx)
            .await?;
    let password_hash = hash_password(&admin_password)?;
    sqlx::query(
        "insert into users (barbershop_id, name, email, password_hash, role) values (?, ?, ?, ?, 'admin')
         on conflict(email) do update set
            barbershop_id = excluded.barbershop_id,
            name = excluded.name,
            password_hash = excluded.password_hash,
            role = excluded.role",
    )
    .bind(barbershop_id.0)
    .bind("Administrador Teste")
    .bind(admin_email)
    .bind(password_hash)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| anyhow::anyhow!("failed to hash seed password"))
}

fn demo_admin_password() -> anyhow::Result<String> {
    #[cfg(test)]
    {
        Ok(std::env::var("DEMO_ADMIN_PASSWORD").unwrap_or_else(|_| "TestPassword@123".to_string()))
    }
    #[cfg(not(test))]
    {
        std::env::var("DEMO_ADMIN_PASSWORD")
            .map_err(|_| anyhow::anyhow!("DEMO_ADMIN_PASSWORD must be set when SEED_DEMO_DATA=1"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Db, migrate, seed_demo_data};
    use sqlx::Executor;

    #[tokio::test]
    async fn sqlite_connections_enable_foreign_key_enforcement() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        let enabled: (i64,) = sqlx::query_as("pragma foreign_keys")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(enabled.0, 1);
    }

    #[tokio::test]
    async fn migrate_adds_tenant_columns_before_creating_indexes_for_legacy_databases() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        db.pool
            .execute(
                "
                create table users (
                    id integer primary key autoincrement,
                    name text not null,
                    email text not null unique,
                    password_hash text not null,
                    role text not null
                );
                create table barbers (
                    id integer primary key autoincrement,
                    name text not null,
                    email text not null,
                    password_hash text not null,
                    specialty text not null default '',
                    status text not null default 'active',
                    monthly_commission_cents integer not null default 0,
                    monthly_tips_cents integer not null default 0,
                    completed_services integer not null default 0,
                    deleted_at text
                );
                ",
            )
            .await
            .unwrap();

        migrate(&db).await.unwrap();

        let columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
            sqlx::query_as("pragma table_info(users)")
                .fetch_all(&db.pool)
                .await
                .unwrap();
        assert!(
            columns
                .iter()
                .any(|(_, name, _, _, _, _)| name == "barbershop_id")
        );

        let barber_columns: Vec<(i64, String, String, i64, Option<String>, i64)> =
            sqlx::query_as("pragma table_info(barbers)")
                .fetch_all(&db.pool)
                .await
                .unwrap();
        assert!(
            barber_columns
                .iter()
                .any(|(_, name, _, _, _, _)| name == "document")
        );
    }

    #[tokio::test]
    async fn migrate_creates_unique_identity_indexes() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        migrate(&db).await.unwrap();

        let indexes: Vec<(String, i64)> = sqlx::query_as(
            "select name, [unique] from pragma_index_list('clients')
             union all
             select name, [unique] from pragma_index_list('barbers')",
        )
        .fetch_all(&db.pool)
        .await
        .unwrap();

        for expected in [
            "idx_clients_active_phone_unique",
            "idx_clients_active_document_unique",
            "idx_barbers_active_email_unique",
            "idx_barbers_active_document",
        ] {
            assert!(
                indexes
                    .iter()
                    .any(|(name, unique)| name == expected && *unique == 1),
                "missing unique index {expected}"
            );
        }
    }

    #[tokio::test]
    async fn demo_seed_creates_login_without_deleting_existing_barbershops() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        migrate(&db).await.unwrap();
        sqlx::query("insert into barbershops (name, slug) values ('Cliente Real', 'cliente-real')")
            .execute(&db.pool)
            .await
            .unwrap();

        seed_demo_data(&db).await.unwrap();

        let existing: (i64,) =
            sqlx::query_as("select count(*) from barbershops where slug = 'cliente-real'")
                .fetch_one(&db.pool)
                .await
                .unwrap();
        let demo: (i64,) =
            sqlx::query_as("select count(*) from users where email = 'admin@example.test'")
                .fetch_one(&db.pool)
                .await
                .unwrap();

        assert_eq!(existing.0, 1);
        assert_eq!(demo.0, 1);
    }

    #[tokio::test]
    async fn demo_seed_works_with_legacy_admin_role_constraint() {
        let db = Db::connect("sqlite::memory:").await.unwrap();
        db.pool
            .execute(
                "
                create table users (
                    id integer primary key autoincrement,
                    name text not null,
                    email text not null unique,
                    password_hash text not null,
                    role text not null check (role in ('admin', 'barber', 'reception'))
                );
                ",
            )
            .await
            .unwrap();
        migrate(&db).await.unwrap();

        seed_demo_data(&db).await.unwrap();
    }
}
