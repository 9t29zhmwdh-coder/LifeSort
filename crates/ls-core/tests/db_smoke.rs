//! Runtime check that the database layer still works, not just compiles.
//!
//! A version bump of sqlx can leave every call site type-correct while
//! changing how migrations are applied or how SQLite values map back into
//! Rust. Opening a fresh database exercises both.

use std::path::PathBuf;

#[tokio::test]
async fn open_applies_migrations_and_serves_queries() {
    let dir = std::env::temp_dir().join(format!("lifesort-db-smoke-{}", std::process::id()));
    let db = dir.join("smoke.db");
    let _ = std::fs::remove_dir_all(&dir);

    let pool = ls_core::db::open(&db).await.expect("open should apply migrations");

    // The migration table is what sqlx itself writes, so a row here proves the
    // migrations ran rather than that the file merely exists.
    let applied: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("migration bookkeeping table should exist");
    assert!(applied > 0, "expected at least one applied migration, got {applied}");

    // Running open a second time must be a no-op rather than an error, which is
    // what every start after the first one does.
    drop(pool);
    let pool = ls_core::db::open(&db).await.expect("reopening should not re-apply migrations");
    let again: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("query after reopen");
    assert_eq!(applied, again, "reopening changed the migration count");

    let _: PathBuf = db;
    let _ = std::fs::remove_dir_all(&dir);
}
