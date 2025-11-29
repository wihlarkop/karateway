use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create config snapshot function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION create_config_snapshot(
                p_version_name VARCHAR(100),
                p_description TEXT DEFAULT NULL,
                p_created_by VARCHAR(100) DEFAULT NULL
            )
            RETURNS UUID AS $$
            DECLARE
                v_snapshot_id UUID;
                v_snapshot JSONB;
            BEGIN
                -- Build complete config snapshot
                SELECT jsonb_build_object(
                    'backend_services', (SELECT jsonb_agg(row_to_json(t.*)) FROM backend_services t WHERE is_active = true),
                    'api_routes', (SELECT jsonb_agg(row_to_json(t.*)) FROM api_routes t WHERE is_active = true),
                    'whitelist_rules', (SELECT jsonb_agg(row_to_json(t.*)) FROM whitelist_rules t WHERE is_active = true),
                    'rate_limits', (SELECT jsonb_agg(row_to_json(t.*)) FROM rate_limits t WHERE is_active = true),
                    'load_balancer_config', (SELECT jsonb_agg(row_to_json(t.*)) FROM load_balancer_config t)
                ) INTO v_snapshot;

                -- Insert snapshot
                INSERT INTO config_versions (version_name, description, config_snapshot, created_by)
                VALUES (p_version_name, p_description, v_snapshot, p_created_by)
                RETURNING id INTO v_snapshot_id;

                RETURN v_snapshot_id;
            END;
            $$ LANGUAGE plpgsql;
            "#
        ).await?;

        // Create notification function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION notify_config_change()
            RETURNS TRIGGER AS $$
            BEGIN
                PERFORM pg_notify('config_update', json_build_object(
                    'table', TG_TABLE_NAME,
                    'operation', TG_OP,
                    'timestamp', CURRENT_TIMESTAMP
                )::text);
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
            "#,
        )
        .await?;

        // Create notification triggers
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("notify_{}_change", table);

            // Drop existing trigger first
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;

            // Create new trigger
            db.execute_unprepared(&format!(
                "CREATE TRIGGER {} AFTER INSERT OR UPDATE OR DELETE ON {} FOR EACH ROW EXECUTE FUNCTION notify_config_change();",
                trigger_name, table
            )).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop all notification triggers
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("notify_{}_change", table);
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;
        }

        // Drop functions
        db.execute_unprepared("DROP FUNCTION IF EXISTS notify_config_change();")
            .await?;
        db.execute_unprepared(
            "DROP FUNCTION IF EXISTS create_config_snapshot(VARCHAR, TEXT, VARCHAR);",
        )
        .await?;

        Ok(())
    }
}
