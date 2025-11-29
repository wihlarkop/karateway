use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Note: Some indexes are already created inline with table definitions
        // We create additional indexes here for performance

        let db = manager.get_connection();

        // api_routes indexes (unique index already created in table definition)
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_api_routes_backend_service ON api_routes(backend_service_id);"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_api_routes_active ON api_routes(is_active);",
        )
        .await?;

        // whitelist_rules indexes
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_whitelist_rules_route ON whitelist_rules(api_route_id);"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_whitelist_rules_active ON whitelist_rules(is_active);",
        )
        .await?;

        // rate_limits indexes
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_rate_limits_route ON rate_limits(api_route_id);",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_rate_limits_active ON rate_limits(is_active);",
        )
        .await?;

        // config_audit_log indexes
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_config_audit_log_table_record ON config_audit_log(table_name, record_id);"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_config_audit_log_timestamp ON config_audit_log(changed_at);"
        ).await?;

        // gateway_metrics indexes
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gateway_metrics_timestamp ON gateway_metrics(timestamp);"
        ).await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gateway_metrics_route ON gateway_metrics(route_id);",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_gateway_metrics_status ON gateway_metrics(status_code);"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP INDEX IF EXISTS idx_api_routes_backend_service;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_api_routes_active;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_whitelist_rules_route;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_whitelist_rules_active;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_rate_limits_route;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_rate_limits_active;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_config_audit_log_table_record;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_config_audit_log_timestamp;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gateway_metrics_timestamp;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gateway_metrics_route;")
            .await?;
        db.execute_unprepared("DROP INDEX IF EXISTS idx_gateway_metrics_status;")
            .await?;

        Ok(())
    }
}
