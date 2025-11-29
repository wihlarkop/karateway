use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LoadBalancerConfig::Table)
                    .if_not_exists()
                    .col(
                        uuid(LoadBalancerConfig::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        uuid(LoadBalancerConfig::BackendServiceId)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        string_len(LoadBalancerConfig::Algorithm, 50)
                            .not_null()
                            .check(Expr::col(LoadBalancerConfig::Algorithm).is_in([
                                "round_robin",
                                "least_conn",
                                "ip_hash",
                                "weighted",
                            ])),
                    )
                    .col(boolean(LoadBalancerConfig::HealthCheckEnabled).default(true))
                    .col(json_binary(LoadBalancerConfig::Config).default("'{}'"))
                    .col(
                        timestamp_with_time_zone(LoadBalancerConfig::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        timestamp_with_time_zone(LoadBalancerConfig::UpdatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_load_balancer_config_backend_service")
                            .from(
                                LoadBalancerConfig::Table,
                                LoadBalancerConfig::BackendServiceId,
                            )
                            .to(BackendServices::Table, BackendServices::Id)
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
                    .table(LoadBalancerConfig::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum LoadBalancerConfig {
    Table,
    Id,
    BackendServiceId,
    Algorithm,
    HealthCheckEnabled,
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BackendServices {
    Table,
    Id,
}
