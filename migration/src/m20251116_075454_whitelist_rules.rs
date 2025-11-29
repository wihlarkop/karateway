use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WhitelistRules::Table)
                    .if_not_exists()
                    .col(
                        uuid(WhitelistRules::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        string_len(WhitelistRules::RuleName, 100)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        string_len(WhitelistRules::RuleType, 50).not_null().check(
                            Expr::col(WhitelistRules::RuleType)
                                .is_in(["ip", "api_key", "jwt", "custom"]),
                        ),
                    )
                    .col(uuid_null(WhitelistRules::ApiRouteId))
                    .col(json_binary(WhitelistRules::Config).not_null())
                    .col(boolean(WhitelistRules::IsActive).default(true))
                    .col(integer(WhitelistRules::Priority).default(0))
                    .col(
                        timestamp_with_time_zone(WhitelistRules::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        timestamp_with_time_zone(WhitelistRules::UpdatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_whitelist_rules_api_route")
                            .from(WhitelistRules::Table, WhitelistRules::ApiRouteId)
                            .to(ApiRoutes::Table, ApiRoutes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(WhitelistRules::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum WhitelistRules {
    Table,
    Id,
    RuleName,
    RuleType,
    ApiRouteId,
    Config,
    IsActive,
    Priority,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiRoutes {
    Table,
    Id,
}
