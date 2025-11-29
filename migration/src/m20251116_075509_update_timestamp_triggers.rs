use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Create the trigger function
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = CURRENT_TIMESTAMP;
                RETURN NEW;
            END;
            $$ language 'plpgsql';
            "#,
        )
        .await?;

        // Create triggers for each table
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("update_{}_updated_at", table);

            // Drop existing triggers to avoid duplicates
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;

            // Create trigger
            db.execute_unprepared(&format!(
                "CREATE TRIGGER {} BEFORE UPDATE ON {} FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();",
                trigger_name, table
            )).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Drop all triggers
        let tables = vec![
            "backend_services",
            "api_routes",
            "whitelist_rules",
            "rate_limits",
            "load_balancer_config",
        ];

        for table in tables {
            let trigger_name = format!("update_{}_updated_at", table);
            db.execute_unprepared(&format!(
                "DROP TRIGGER IF EXISTS {} ON {};",
                trigger_name, table
            ))
            .await?;
        }

        // Drop the function
        db.execute_unprepared("DROP FUNCTION IF EXISTS update_updated_at_column();")
            .await?;

        Ok(())
    }
}
