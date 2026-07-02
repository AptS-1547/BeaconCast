//! Migration smoke tests.

use sea_orm::Database;
use sea_orm_migration::prelude::MigratorTrait;
use sea_orm_migration::prelude::SchemaManager;

#[test]
fn foundation_migration_is_registered_first() {
    let migrations = migration::Migrator::migrations();

    assert_eq!(migrations.len(), 4);
    assert_eq!(
        migrations[0].name(),
        "m20260627_000001_forge_foundation_schema"
    );
    assert_eq!(
        migrations[1].name(),
        "m20260701_000002_beacon_cast_product_schema"
    );
    assert_eq!(
        migrations[2].name(),
        "m20260702_000003_activity_usage_spans"
    );
    assert_eq!(
        migrations[3].name(),
        "m20260702_000004_activity_classification_config"
    );
}

#[tokio::test]
async fn migrations_apply_and_roll_back_on_sqlite() {
    let database_url = format!("sqlite://{}?mode=rwc", unique_database_path().display());
    let db = Database::connect(&database_url)
        .await
        .expect("connect sqlite migration test database");

    migration::Migrator::up(&db, None)
        .await
        .expect("apply foundation migration");
    let manager = SchemaManager::new(&db);

    for table in [
        "runtime_leases",
        "scheduled_tasks",
        "system_config",
        "mail_outbox",
        "audit_logs",
        "admin_users",
        "admin_sessions",
        "beacon_devices",
        "beacon_device_tokens",
        "activity_events",
        "activity_usage_spans",
        "manual_overrides",
        "activity_actions",
        "activity_applications",
        "activity_application_aliases",
    ] {
        assert!(
            manager
                .has_table(table)
                .await
                .expect("query migrated table"),
            "expected {table} to be created"
        );
    }

    migration::Migrator::down(&db, None)
        .await
        .expect("roll back foundation migration");
    for table in [
        "runtime_leases",
        "scheduled_tasks",
        "system_config",
        "mail_outbox",
        "audit_logs",
        "admin_users",
        "admin_sessions",
        "beacon_devices",
        "beacon_device_tokens",
        "activity_events",
        "activity_usage_spans",
        "manual_overrides",
        "activity_actions",
        "activity_applications",
        "activity_application_aliases",
    ] {
        assert!(
            !manager
                .has_table(table)
                .await
                .expect("query rolled back table"),
            "expected {table} to be dropped"
        );
    }
}

fn unique_database_path() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{}-migration-{nanos}.db", env!("CARGO_PKG_NAME")))
}
