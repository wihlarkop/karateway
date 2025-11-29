use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create the audit trigger function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION audit_trigger_func()
            RETURNS TRIGGER AS $$
            BEGIN
                IF (TG_OP = 'DELETE') THEN
                    INSERT INTO config_audit_log (table_name, record_id, operation, old_data, changed_by)
                    VALUES (TG_TABLE_NAME, OLD.id, TG_OP, row_to_json(OLD), current_user);
                    RETURN OLD;
                ELSIF (TG_OP = 'UPDATE') THEN
                    INSERT INTO config_audit_log (table_name, record_id, operation, old_data, new_data, changed_by)
                    VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(OLD), row_to_json(NEW), current_user);
                    RETURN NEW;
                ELSIF (TG_OP = 'INSERT') THEN
                    INSERT INTO config_audit_log (table_name, record_id, operation, new_data, changed_by)
                    VALUES (TG_TABLE_NAME, NEW.id, TG_OP, row_to_json(NEW), current_user);
                    RETURN NEW;
                END IF;
                RETURN NULL;
            END;
            $$ LANGUAGE plpgsql;
            "#
        ).await?;

        // Create audit triggers for each configuration table
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("audit_{}", table);

            // Drop existing trigger first
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;

            // Create new trigger
            db.execute_unprepared(&format!(
                "CREATE TRIGGER {} AFTER INSERT OR UPDATE OR DELETE ON {} FOR EACH ROW EXECUTE FUNCTION audit_trigger_func();",
                trigger_name, table
            )).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop all audit triggers
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("audit_{}", table);
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;
        }

        // Drop the function
        db.execute_unprepared("DROP FUNCTION IF EXISTS audit_trigger_func();")
            .await?;

        Ok(())
    }
}
