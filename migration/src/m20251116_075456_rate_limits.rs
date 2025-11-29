use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RateLimits::Table)
                    .if_not_exists()
                    .col(
                        uuid(RateLimits::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(string_len(RateLimits::Name, 100).not_null().unique_key())
                    .col(uuid_null(RateLimits::ApiRouteId))
                    .col(integer(RateLimits::MaxRequests).not_null())
                    .col(integer(RateLimits::WindowSeconds).not_null())
                    .col(
                        string_len(RateLimits::IdentifierType, 50).not_null().check(
                            Expr::col(RateLimits::IdentifierType)
                                .is_in(["ip", "api_key", "user_id", "global"]),
                        ),
                    )
                    .col(boolean(RateLimits::IsActive).default(true))
                    .col(integer_null(RateLimits::BurstSize))
                    .col(
                        timestamp_with_time_zone(RateLimits::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        timestamp_with_time_zone(RateLimits::UpdatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_rate_limits_api_route")
                            .from(RateLimits::Table, RateLimits::ApiRouteId)
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
                    .table(RateLimits::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RateLimits {
    Table,
    Id,
    Name,
    ApiRouteId,
    MaxRequests,
    WindowSeconds,
    IdentifierType,
    IsActive,
    BurstSize,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ApiRoutes {
    Table,
    Id,
}
