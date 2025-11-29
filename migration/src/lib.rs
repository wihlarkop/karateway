pub use sea_orm_migration::prelude::*;

mod m20251116_075450_backend_services;
mod m20251116_075452_api_routes;
mod m20251116_075454_whitelist_rules;
mod m20251116_075456_rate_limits;
mod m20251116_075458_load_balancer_config;
mod m20251116_075501_config_audit_log;
mod m20251116_075503_config_versions;
mod m20251116_075505_gateway_metrics;
mod m20251116_075507_indexes;
mod m20251116_075509_update_timestamp_triggers;
mod m20251116_075511_audit_triggers;
mod m20251116_075513_config_snapshot_functions;
mod m20251116_075515_audit_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251116_075450_backend_services::Migration),
            Box::new(m20251116_075452_api_routes::Migration),
            Box::new(m20251116_075454_whitelist_rules::Migration),
            Box::new(m20251116_075456_rate_limits::Migration),
            Box::new(m20251116_075458_load_balancer_config::Migration),
            Box::new(m20251116_075501_config_audit_log::Migration),
            Box::new(m20251116_075503_config_versions::Migration),
            Box::new(m20251116_075505_gateway_metrics::Migration),
            Box::new(m20251116_075507_indexes::Migration),
            Box::new(m20251116_075509_update_timestamp_triggers::Migration),
            Box::new(m20251116_075511_audit_triggers::Migration),
            Box::new(m20251116_075513_config_snapshot_functions::Migration),
            Box::new(m20251116_075515_audit_logs::Migration),
        ]
    }
}
