//! Agent-reported usage span ledger.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ActivityUsageSpans::Table)
                    .if_not_exists()
                    .col(pk(ActivityUsageSpans::Id))
                    .col(bigint(ActivityUsageSpans::DeviceId).not_null())
                    .col(timestamp(ActivityUsageSpans::StartedAt).not_null())
                    .col(timestamp(ActivityUsageSpans::EndedAt).not_null())
                    .col(bigint(ActivityUsageSpans::DurationSeconds).not_null())
                    .col(string(ActivityUsageSpans::AppLabel, 128).null())
                    .col(string(ActivityUsageSpans::ProjectKey, 128).null())
                    .col(string(ActivityUsageSpans::ProjectLabel, 128).null())
                    .col(boolean(ActivityUsageSpans::Idle).not_null().default(false))
                    .col(string(ActivityUsageSpans::Source, 64).not_null())
                    .col(
                        float(ActivityUsageSpans::Confidence)
                            .not_null()
                            .default(1.0),
                    )
                    .col(text(ActivityUsageSpans::MetadataJson).not_null())
                    .col(timestamp(ActivityUsageSpans::CreatedAt).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                index(
                    "idx_activity_usage_spans_started_at",
                    ActivityUsageSpans::Table,
                    ActivityUsageSpans::StartedAt,
                )
                .to_owned(),
            )
            .await?;
        manager
            .create_index(
                index(
                    "idx_activity_usage_spans_device_started",
                    ActivityUsageSpans::Table,
                    ActivityUsageSpans::DeviceId,
                )
                .col(ActivityUsageSpans::StartedAt)
                .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ActivityUsageSpans::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

fn pk<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.big_integer().not_null().auto_increment().primary_key();
    def
}

fn bigint<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.big_integer();
    def
}

fn boolean<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.boolean();
    def
}

fn float<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.float();
    def
}

fn string<T>(column: T, len: u32) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.string_len(len);
    def
}

fn text<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.text();
    def
}

fn timestamp<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.timestamp_with_time_zone();
    def
}

fn index<C>(name: &str, table: impl IntoIden, column: C) -> IndexCreateStatement
where
    C: IntoIden,
{
    Index::create()
        .name(name)
        .table(table)
        .col(column)
        .if_not_exists()
        .to_owned()
}

#[derive(DeriveIden)]
enum ActivityUsageSpans {
    Table,
    Id,
    DeviceId,
    StartedAt,
    EndedAt,
    DurationSeconds,
    AppLabel,
    ProjectKey,
    ProjectLabel,
    Idle,
    Source,
    Confidence,
    MetadataJson,
    CreatedAt,
}
