use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConfigVersions::Table)
                    .if_not_exists()
                    .col(
                        uuid(ConfigVersions::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(
                        string_len(ConfigVersions::VersionName, 100)
                            .not_null()
                            .unique_key(),
                    )
                    .col(text_null(ConfigVersions::Description))
                    .col(json_binary(ConfigVersions::ConfigSnapshot).not_null())
                    .col(string_len_null(ConfigVersions::CreatedBy, 100))
                    .col(
                        timestamp_with_time_zone(ConfigVersions::CreatedAt)
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
                    .table(ConfigVersions::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ConfigVersions {
    Table,
    Id,
    VersionName,
    Description,
    ConfigSnapshot,
    CreatedBy,
    CreatedAt,
}
