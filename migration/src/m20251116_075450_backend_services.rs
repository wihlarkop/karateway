use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BackendServices::Table)
                    .if_not_exists()
                    .col(
                        uuid(BackendServices::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        string_len(BackendServices::Name, 100)
                            .not_null()
                            .unique_key(),
                    )
                    .col(text_null(BackendServices::Description))
                    .col(string_len(BackendServices::BaseUrl, 500).not_null())
                    .col(string_len_null(BackendServices::HealthCheckUrl, 500))
                    .col(integer(BackendServices::HealthCheckIntervalSeconds).default(30))
                    .col(integer(BackendServices::TimeoutMs).default(5000))
                    .col(boolean(BackendServices::IsActive).default(true))
                    .col(
                        timestamp_with_time_zone(BackendServices::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .col(
                        timestamp_with_time_zone(BackendServices::UpdatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(BackendServices::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BackendServices {
    Table,
    Id,
    Name,
    Description,
    BaseUrl,
    HealthCheckUrl,
    HealthCheckIntervalSeconds,
    TimeoutMs,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
