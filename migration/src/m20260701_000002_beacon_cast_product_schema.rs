//! BeaconCast product tables.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_admin_users(manager).await?;
        create_admin_sessions(manager).await?;
        create_beacon_devices(manager).await?;
        create_beacon_device_tokens(manager).await?;
        create_activity_events(manager).await?;
        create_manual_overrides(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ManualOverrides::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ActivityEvents::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(BeaconDeviceTokens::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(BeaconDevices::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AdminSessions::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(AdminUsers::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

async fn create_admin_users(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AdminUsers::Table)
                .if_not_exists()
                .col(pk(AdminUsers::Id))
                .col(string(AdminUsers::Username, 128).not_null().unique_key())
                .col(string(AdminUsers::PasswordHash, 512).not_null())
                .col(string(AdminUsers::DisplayName, 128).not_null())
                .col(timestamp(AdminUsers::LastLoginAt).null())
                .col(timestamp(AdminUsers::DisabledAt).null())
                .col(timestamp(AdminUsers::CreatedAt).not_null())
                .col(timestamp(AdminUsers::UpdatedAt).not_null())
                .to_owned(),
        )
        .await
}

async fn create_admin_sessions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(AdminSessions::Table)
                .if_not_exists()
                .col(pk(AdminSessions::Id))
                .col(bigint(AdminSessions::UserId).not_null())
                .col(
                    string(AdminSessions::TokenHash, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(timestamp(AdminSessions::ExpiresAt).not_null())
                .col(timestamp(AdminSessions::RevokedAt).null())
                .col(timestamp(AdminSessions::CreatedAt).not_null())
                .col(timestamp(AdminSessions::LastSeenAt).null())
                .to_owned(),
        )
        .await?;
    manager
        .create_index(index(
            "idx_admin_sessions_user_id",
            AdminSessions::Table,
            AdminSessions::UserId,
        ))
        .await
}

async fn create_beacon_devices(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(BeaconDevices::Table)
                .if_not_exists()
                .col(pk(BeaconDevices::Id))
                .col(
                    string(BeaconDevices::DeviceKey, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(string(BeaconDevices::DisplayName, 128).not_null())
                .col(string(BeaconDevices::Kind, 64).not_null())
                .col(integer(BeaconDevices::Priority).not_null().default(0))
                .col(text(BeaconDevices::CapabilitiesJson).not_null())
                .col(timestamp(BeaconDevices::LastSeenAt).null())
                .col(timestamp(BeaconDevices::DisabledAt).null())
                .col(timestamp(BeaconDevices::CreatedAt).not_null())
                .col(timestamp(BeaconDevices::UpdatedAt).not_null())
                .to_owned(),
        )
        .await
}

async fn create_beacon_device_tokens(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(BeaconDeviceTokens::Table)
                .if_not_exists()
                .col(pk(BeaconDeviceTokens::Id))
                .col(bigint(BeaconDeviceTokens::DeviceId).not_null())
                .col(string(BeaconDeviceTokens::Name, 128).not_null())
                .col(
                    string(BeaconDeviceTokens::TokenHash, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(timestamp(BeaconDeviceTokens::LastUsedAt).null())
                .col(timestamp(BeaconDeviceTokens::RevokedAt).null())
                .col(timestamp(BeaconDeviceTokens::CreatedAt).not_null())
                .to_owned(),
        )
        .await?;
    manager
        .create_index(index(
            "idx_beacon_device_tokens_device_id",
            BeaconDeviceTokens::Table,
            BeaconDeviceTokens::DeviceId,
        ))
        .await
}

async fn create_activity_events(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ActivityEvents::Table)
                .if_not_exists()
                .col(pk(ActivityEvents::Id))
                .col(bigint(ActivityEvents::DeviceId).not_null())
                .col(timestamp(ActivityEvents::ObservedAt).not_null())
                .col(string(ActivityEvents::AppLabel, 128).null())
                .col(string(ActivityEvents::ProjectKey, 128).null())
                .col(string(ActivityEvents::ProjectLabel, 128).null())
                .col(boolean(ActivityEvents::Idle).not_null().default(false))
                .col(string(ActivityEvents::Source, 64).not_null())
                .col(float(ActivityEvents::Confidence).not_null().default(1.0))
                .col(text(ActivityEvents::MetadataJson).not_null())
                .col(timestamp(ActivityEvents::CreatedAt).not_null())
                .to_owned(),
        )
        .await?;
    manager
        .create_index(index(
            "idx_activity_events_observed_at",
            ActivityEvents::Table,
            ActivityEvents::ObservedAt,
        ))
        .await?;
    manager
        .create_index(
            index(
                "idx_activity_events_device_observed",
                ActivityEvents::Table,
                ActivityEvents::DeviceId,
            )
            .col(ActivityEvents::ObservedAt)
            .to_owned(),
        )
        .await
}

async fn create_manual_overrides(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ManualOverrides::Table)
                .if_not_exists()
                .col(pk(ManualOverrides::Id))
                .col(string(ManualOverrides::Status, 32).not_null())
                .col(string(ManualOverrides::ActivityLabel, 255).not_null())
                .col(timestamp(ManualOverrides::StartsAt).not_null())
                .col(timestamp(ManualOverrides::ExpiresAt).null())
                .col(timestamp(ManualOverrides::ClearedAt).null())
                .col(timestamp(ManualOverrides::CreatedAt).not_null())
                .col(bigint(ManualOverrides::CreatedBy).null())
                .to_owned(),
        )
        .await?;
    manager
        .create_index(index(
            "idx_manual_overrides_active",
            ManualOverrides::Table,
            ManualOverrides::ClearedAt,
        ))
        .await
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

fn integer<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.integer();
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

fn boolean<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.boolean();
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
enum AdminUsers {
    Table,
    Id,
    Username,
    PasswordHash,
    DisplayName,
    LastLoginAt,
    DisabledAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum AdminSessions {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
    LastSeenAt,
}

#[derive(DeriveIden)]
enum BeaconDevices {
    Table,
    Id,
    DeviceKey,
    DisplayName,
    Kind,
    Priority,
    CapabilitiesJson,
    LastSeenAt,
    DisabledAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BeaconDeviceTokens {
    Table,
    Id,
    DeviceId,
    Name,
    TokenHash,
    LastUsedAt,
    RevokedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ActivityEvents {
    Table,
    Id,
    DeviceId,
    ObservedAt,
    AppLabel,
    ProjectKey,
    ProjectLabel,
    Idle,
    Source,
    Confidence,
    MetadataJson,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ManualOverrides {
    Table,
    Id,
    Status,
    ActivityLabel,
    StartsAt,
    ExpiresAt,
    ClearedAt,
    CreatedAt,
    CreatedBy,
}
