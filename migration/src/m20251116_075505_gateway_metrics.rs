use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GatewayMetrics::Table)
                    .if_not_exists()
                    .col(
                        uuid(GatewayMetrics::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        timestamp_with_time_zone(GatewayMetrics::Timestamp)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(uuid_null(GatewayMetrics::RouteId))
                    .col(string_len_null(GatewayMetrics::Method, 10))
                    .col(string_len_null(GatewayMetrics::Path, 500))
                    .col(integer_null(GatewayMetrics::StatusCode))
                    .col(float_null(GatewayMetrics::ResponseTimeMs))
                    .col(uuid_null(GatewayMetrics::BackendServiceId))
                    .col(text_null(GatewayMetrics::ErrorMessage))
                    .col(json_binary(GatewayMetrics::Metadata).default("{}"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_gateway_metrics_route")
                            .from(GatewayMetrics::Table, GatewayMetrics::RouteId)
                            .to(ApiRoutes::Table, ApiRoutes::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_gateway_metrics_backend_service")
                            .from(GatewayMetrics::Table, GatewayMetrics::BackendServiceId)
                            .to(BackendServices::Table, BackendServices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(GatewayMetrics::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GatewayMetrics {
    Table,
    Id,
    Timestamp,
    RouteId,
    Method,
    Path,
    StatusCode,
    ResponseTimeMs,
    BackendServiceId,
    ErrorMessage,
    Metadata,
}

#[derive(DeriveIden)]
enum ApiRoutes {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum BackendServices {
    Table,
    Id,
}
