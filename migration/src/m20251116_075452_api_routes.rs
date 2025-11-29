use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ApiRoutes::Table)
                    .if_not_exists()
                    .col(
                        uuid(ApiRoutes::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(string_len(ApiRoutes::PathPattern, 500).not_null())
                    .col(
                        string_len(ApiRoutes::Method, 10).not_null().check(
                            Expr::col(ApiRoutes::Method).is_in([
                                "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS",
                            ]),
                        ),
                    )
                    .col(uuid(ApiRoutes::BackendServiceId).not_null())
                    .col(boolean(ApiRoutes::StripPathPrefix).default(false))
                    .col(boolean(ApiRoutes::PreserveHostHeader).default(false))
                    .col(integer(ApiRoutes::TimeoutMs).default(30000))
                    .col(boolean(ApiRoutes::IsActive).default(true))
                    .col(integer(ApiRoutes::Priority).default(0))
                    .col(json_binary(ApiRoutes::Metadata).default("'{}'"))
                    .col(
                        timestamp_with_time_zone(ApiRoutes::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        timestamp_with_time_zone(ApiRoutes::UpdatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_api_routes_backend_service")
                            .from(ApiRoutes::Table, ApiRoutes::BackendServiceId)
                            .to(BackendServices::Table, BackendServices::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_api_routes_path_method")
                            .col(ApiRoutes::PathPattern)
                            .col(ApiRoutes::Method),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ApiRoutes::Table).if_exists().to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ApiRoutes {
    Table,
    Id,
    PathPattern,
    Method,
    BackendServiceId,
    StripPathPrefix,
    PreserveHostHeader,
    TimeoutMs,
    IsActive,
    Priority,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BackendServices {
    Table,
    Id,
}
