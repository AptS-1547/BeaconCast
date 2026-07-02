//! Server-owned activity classification tables.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_activity_actions(manager).await?;
        create_activity_applications(manager).await?;
        create_activity_application_aliases(manager).await?;
        seed_default_actions(manager).await?;
        seed_default_applications(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ActivityApplicationAliases::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ActivityApplications::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ActivityActions::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

async fn create_activity_actions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ActivityActions::Table)
                .if_not_exists()
                .col(pk(ActivityActions::Id))
                .col(
                    string(ActivityActions::ActionKey, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(string(ActivityActions::Label, 128).not_null())
                .col(string(ActivityActions::Status, 32).not_null())
                .col(string(ActivityActions::Category, 64).not_null())
                .col(string(ActivityActions::PublicLabel, 128).not_null())
                .col(string(ActivityActions::MessageTemplate, 255).not_null())
                .col(boolean(ActivityActions::Enabled).not_null().default(true))
                .col(integer(ActivityActions::SortOrder).not_null().default(0))
                .col(timestamp(ActivityActions::CreatedAt).not_null())
                .col(timestamp(ActivityActions::UpdatedAt).not_null())
                .to_owned(),
        )
        .await
}

async fn create_activity_applications(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ActivityApplications::Table)
                .if_not_exists()
                .col(pk(ActivityApplications::Id))
                .col(
                    string(ActivityApplications::AppKey, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(string(ActivityApplications::DisplayName, 128).not_null())
                .col(bigint(ActivityApplications::DefaultActionId).null())
                .col(
                    boolean(ActivityApplications::Enabled)
                        .not_null()
                        .default(true),
                )
                .col(timestamp(ActivityApplications::CreatedAt).not_null())
                .col(timestamp(ActivityApplications::UpdatedAt).not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_activity_applications_default_action")
                        .from(
                            ActivityApplications::Table,
                            ActivityApplications::DefaultActionId,
                        )
                        .to(ActivityActions::Table, ActivityActions::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .to_owned(),
        )
        .await
}

async fn create_activity_application_aliases(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(ActivityApplicationAliases::Table)
                .if_not_exists()
                .col(pk(ActivityApplicationAliases::Id))
                .col(bigint(ActivityApplicationAliases::ApplicationId).not_null())
                .col(string(ActivityApplicationAliases::Alias, 128).not_null())
                .col(
                    string(ActivityApplicationAliases::NormalizedAlias, 128)
                        .not_null()
                        .unique_key(),
                )
                .col(timestamp(ActivityApplicationAliases::CreatedAt).not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_activity_application_aliases_app")
                        .from(
                            ActivityApplicationAliases::Table,
                            ActivityApplicationAliases::ApplicationId,
                        )
                        .to(ActivityApplications::Table, ActivityApplications::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .name("idx_activity_application_aliases_app")
                .table(ActivityApplicationAliases::Table)
                .col(ActivityApplicationAliases::ApplicationId)
                .if_not_exists()
                .to_owned(),
        )
        .await
}

async fn seed_default_actions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for action in [
        (
            "writing_code",
            "Writing code",
            "coding",
            "coding",
            "Writing code",
            "Writing code",
            10,
        ),
        (
            "communicating",
            "Communicating",
            "communicating",
            "communication",
            "Communicating",
            "Talking with people",
            20,
        ),
        (
            "reading_reference",
            "Reading reference",
            "reading",
            "research",
            "Reading",
            "Reading reference material",
            30,
        ),
        (
            "running_commands",
            "Running commands",
            "coding",
            "coding",
            "Running commands",
            "Running development commands",
            40,
        ),
    ] {
        insert_action(manager, action).await?;
    }
    Ok(())
}

async fn seed_default_applications(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for app in [
        (
            "code",
            "Code",
            "writing_code",
            &["Code", "Visual Studio Code", "Cursor", "Zed"][..],
        ),
        (
            "wechat",
            "WeChat",
            "communicating",
            &["WeChat", "Weixin", "微信"][..],
        ),
        (
            "terminal",
            "Terminal",
            "running_commands",
            &["Terminal", "iTerm", "iTerm2", "Warp"][..],
        ),
        (
            "browser",
            "Browser",
            "reading_reference",
            &[
                "Safari",
                "Arc",
                "Google Chrome",
                "Chrome",
                "Firefox",
                "Microsoft Edge",
            ][..],
        ),
    ] {
        insert_application(manager, app).await?;
    }
    Ok(())
}

async fn insert_action(
    manager: &SchemaManager<'_>,
    action: (&str, &str, &str, &str, &str, &str, i32),
) -> Result<(), DbErr> {
    let now = Expr::current_timestamp();
    let mut insert = Query::insert();
    insert
        .into_table(ActivityActions::Table)
        .columns([
            ActivityActions::ActionKey,
            ActivityActions::Label,
            ActivityActions::Status,
            ActivityActions::Category,
            ActivityActions::PublicLabel,
            ActivityActions::MessageTemplate,
            ActivityActions::Enabled,
            ActivityActions::SortOrder,
            ActivityActions::CreatedAt,
            ActivityActions::UpdatedAt,
        ])
        .values_panic([
            action.0.into(),
            action.1.into(),
            action.2.into(),
            action.3.into(),
            action.4.into(),
            action.5.into(),
            true.into(),
            action.6.into(),
            now.clone().into(),
            now.into(),
        ])
        .on_conflict(
            OnConflict::column(ActivityActions::ActionKey)
                .do_nothing()
                .to_owned(),
        );
    manager.execute(insert.to_owned()).await?;
    Ok(())
}

async fn insert_application(
    manager: &SchemaManager<'_>,
    app: (&str, &str, &str, &[&str]),
) -> Result<(), DbErr> {
    let now = Expr::current_timestamp();
    let action_id_expr = Query::select()
        .column(ActivityActions::Id)
        .from(ActivityActions::Table)
        .and_where(Expr::col(ActivityActions::ActionKey).eq(app.2))
        .take();
    let mut app_insert = Query::insert();
    app_insert
        .into_table(ActivityApplications::Table)
        .columns([
            ActivityApplications::AppKey,
            ActivityApplications::DisplayName,
            ActivityApplications::DefaultActionId,
            ActivityApplications::Enabled,
            ActivityApplications::CreatedAt,
            ActivityApplications::UpdatedAt,
        ])
        .values_panic([
            app.0.into(),
            app.1.into(),
            SimpleExpr::SubQuery(
                None,
                Box::new(SubQueryStatement::SelectStatement(action_id_expr)),
            ),
            true.into(),
            now.clone().into(),
            now.clone().into(),
        ])
        .on_conflict(
            OnConflict::column(ActivityApplications::AppKey)
                .do_nothing()
                .to_owned(),
        );
    manager.execute(app_insert.to_owned()).await?;

    let application_id = Query::select()
        .column(ActivityApplications::Id)
        .from(ActivityApplications::Table)
        .and_where(Expr::col(ActivityApplications::AppKey).eq(app.0))
        .take();
    for alias in app.3 {
        let mut alias_insert = Query::insert();
        alias_insert
            .into_table(ActivityApplicationAliases::Table)
            .columns([
                ActivityApplicationAliases::ApplicationId,
                ActivityApplicationAliases::Alias,
                ActivityApplicationAliases::NormalizedAlias,
                ActivityApplicationAliases::CreatedAt,
            ])
            .values_panic([
                SimpleExpr::SubQuery(
                    None,
                    Box::new(SubQueryStatement::SelectStatement(application_id.clone())),
                ),
                (*alias).into(),
                normalize_alias(alias).into(),
                now.clone().into(),
            ])
            .on_conflict(
                OnConflict::column(ActivityApplicationAliases::NormalizedAlias)
                    .do_nothing()
                    .to_owned(),
            );
        manager.execute(alias_insert.to_owned()).await?;
    }
    Ok(())
}

fn normalize_alias(value: &str) -> String {
    value.trim().to_ascii_lowercase()
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

fn timestamp<T>(column: T) -> ColumnDef
where
    T: IntoIden,
{
    let mut def = ColumnDef::new(column);
    def.timestamp_with_time_zone();
    def
}

#[derive(DeriveIden)]
enum ActivityActions {
    Table,
    Id,
    ActionKey,
    Label,
    Status,
    Category,
    PublicLabel,
    MessageTemplate,
    Enabled,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ActivityApplications {
    Table,
    Id,
    AppKey,
    DisplayName,
    DefaultActionId,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ActivityApplicationAliases {
    Table,
    Id,
    ApplicationId,
    Alias,
    NormalizedAlias,
    CreatedAt,
}
