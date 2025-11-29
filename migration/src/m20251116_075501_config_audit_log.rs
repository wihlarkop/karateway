use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConfigAuditLog::Table)
                    .if_not_exists()
                    .col(
                        uuid(ConfigAuditLog::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(string_len(ConfigAuditLog::TableName, 100).not_null())
                    .col(uuid(ConfigAuditLog::RecordId).not_null())
                    .col(string_len(ConfigAuditLog::Operation, 20).not_null().check(
                        Expr::col(ConfigAuditLog::Operation).is_in(["INSERT", "UPDATE", "DELETE"]),
                    ))
                    .col(json_binary_null(ConfigAuditLog::OldData))
                    .col(json_binary_null(ConfigAuditLog::NewData))
                    .col(string_len_null(ConfigAuditLog::ChangedBy, 100))
                    .col(
                        timestamp_with_time_zone(ConfigAuditLog::ChangedAt)
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
                    .table(ConfigAuditLog::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ConfigAuditLog {
    Table,
    Id,
    TableName,
    RecordId,
    Operation,
    OldData,
    NewData,
    ChangedBy,
    ChangedAt,
}
