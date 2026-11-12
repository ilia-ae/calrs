//! ilia fork: what the `config apply` tests share (`apply_*.rs`): a database
//! in a file of its own (the run opens its own one-connection pool on it,
//! as `calrs config apply` does), an enabled admin with an account, a run,
//! an export, and a snapshot of every row.

use super::*;
use crate::web::apply::{self, ApplyError, Document, Report};

/// The admin every `ApplyEnv` has.
pub(super) const ADMIN_EMAIL: &str = "admin@example.com";

/// A data directory with a migrated `calrs.db` and the admin.
pub(super) struct ApplyEnv {
    pub(super) dir: std::path::PathBuf,
    /// A pool as `db::connect` opens one (5 connections), outside the run:
    /// for seeding, and for reading what another process would see.
    pub(super) pool: SqlitePool,
    pub(super) admin_id: String,
    pub(super) account_id: String,
}

impl ApplyEnv {
    pub(super) async fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("calrs-apply-{}", uuid::Uuid::new_v4()));
        let pool = crate::db::connect(&dir).await.unwrap();
        crate::db::migrate(&pool).await.unwrap();
        let admin_id = insert_role_user(&pool, ADMIN_EMAIL, "admin").await;
        let account_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO accounts (id, name, email, timezone, user_id) VALUES (?, 'Admin', ?, 'UTC', ?)",
        )
        .bind(&account_id)
        .bind(ADMIN_EMAIL)
        .bind(&admin_id)
        .execute(&pool)
        .await
        .unwrap();
        Self {
            dir,
            pool,
            admin_id,
            account_id,
        }
    }

    pub(super) fn db_path(&self) -> std::path::PathBuf {
        self.dir.join("calrs.db")
    }

    /// The run's own pool: one connection, as `config apply` opens it.
    pub(super) async fn single(&self, read_only: bool) -> SqlitePool {
        apply::open_pool(&self.db_path(), read_only).await.unwrap()
    }

    /// `config apply --stdin [--dry-run]` with `json` on stdin.
    pub(super) async fn apply(&self, json: &str, dry_run: bool) -> Result<Report, ApplyError> {
        let doc = apply::parse(json)?;
        let pool = self.single(false).await;
        let report = apply::run(pool.clone(), self.dir.clone(), [0u8; 32], &doc, dry_run).await;
        pool.close().await;
        report
    }

    /// `config apply --export`.
    pub(super) async fn export(&self) -> Result<Document, ApplyError> {
        let pool = self.single(true).await;
        let doc = apply::export_document(pool.clone(), self.dir.clone(), [0u8; 32], None).await;
        pool.close().await;
        doc
    }

    /// Every row of every table, as text (`db_snapshot`).
    pub(super) async fn snapshot(&self) -> String {
        db_snapshot(&self.pool).await
    }
}

impl Drop for ApplyEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Every row of every table, each value `quote()`d, in a stable order: two
/// snapshots are equal exactly when the stored data is.
pub(super) async fn db_snapshot(pool: &SqlitePool) -> String {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let mut out = String::new();
    for table in tables {
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?) ORDER BY cid")
                .bind(&table)
                .fetch_all(pool)
                .await
                .unwrap();
        let row = columns
            .iter()
            .map(|c| format!("quote(\"{c}\")"))
            .collect::<Vec<_>>()
            .join(" || '|' || ");
        let rows: Vec<String> =
            sqlx::query_scalar(&format!("SELECT {row} FROM \"{table}\" ORDER BY 1"))
                .fetch_all(pool)
                .await
                .unwrap();
        out.push_str(&format!("## {table}\n"));
        for r in rows {
            out.push_str(&r);
            out.push('\n');
        }
    }
    out
}

/// The smallest valid document for the admin.
pub(super) fn empty_doc() -> String {
    format!(r#"{{"format":"{}","user":"{ADMIN_EMAIL}"}}"#, apply::FORMAT)
}
