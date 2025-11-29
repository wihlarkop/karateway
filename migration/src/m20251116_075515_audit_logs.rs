use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create audit_logs table
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        uuid(AuditLogs::Id)
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()"),
                    )
                    .col(string_len(AuditLogs::EventType, 50).not_null())
                    .col(string_len(AuditLogs::EventCategory, 50).not_null())
                    .col(string_len(AuditLogs::Severity, 20).not_null())
                    .col(string_len_null(AuditLogs::RequestMethod, 10))
                    .col(text_null(AuditLogs::RequestPath))
                    .col(string_len_null(AuditLogs::ClientIp, 45))
                    .col(text_null(AuditLogs::UserAgent))
                    .col(uuid_null(AuditLogs::ApiRouteId))
                    .col(uuid_null(AuditLogs::BackendServiceId))
                    .col(text(AuditLogs::Message).not_null())
                    .col(json_binary(AuditLogs::Metadata).default("'{}'::jsonb"))
                    .col(integer_null(AuditLogs::StatusCode))
                    .col(
                        timestamp_with_time_zone(AuditLogs::CreatedAt)
                            .extra("DEFAULT CURRENT_TIMESTAMP"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_api_route")
                            .from(AuditLogs::Table, AuditLogs::ApiRouteId)
                            .to(ApiRoutes::Table, ApiRoutes::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_backend_service")
                            .from(AuditLogs::Table, AuditLogs::BackendServiceId)
                            .to(BackendServices::Table, BackendServices::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        let db = manager.get_connection();

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_event_category ON audit_logs(event_category);"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_client_ip ON audit_logs(client_ip);",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_audit_logs_severity ON audit_logs(severity);",
        )
        .await?;

        // Create cleanup function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION cleanup_old_audit_logs() RETURNS void AS $$
            BEGIN
                DELETE FROM audit_logs
                WHERE created_at < NOW() - INTERVAL '90 days';
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // Add comments
        db.execute_unprepared(
            "COMMENT ON TABLE audit_logs IS 'Security event audit logs with 90-day retention policy';"
        ).await?;

        db.execute_unprepared(
            "COMMENT ON FUNCTION cleanup_old_audit_logs() IS 'Deletes audit logs older than 90 days. Run manually: SELECT cleanup_old_audit_logs();';"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop function
        db.execute_unprepared("DROP FUNCTION IF EXISTS cleanup_old_audit_logs();")
            .await?;

        // Drop table (indexes will be dropped automatically)
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).if_exists().to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    EventType,
    EventCategory,
    Severity,
    RequestMethod,
    RequestPath,
    ClientIp,
    UserAgent,
    ApiRouteId,
    BackendServiceId,
    Message,
    Metadata,
    StatusCode,
    CreatedAt,
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
