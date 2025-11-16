use dashmap::DashMap;
use karateway_core::models::BackendService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config_loader::ConfigLoader;

/// Health status for a backend service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Health checker for backend services
pub struct HealthChecker {
    /// Map of service_id -> health status
    service_health: Arc<DashMap<Uuid, HealthStatus>>,
    /// Configuration loader
    config_loader: Arc<ConfigLoader>,
    /// HTTP client for health checks
    client: reqwest::Client,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config_loader: Arc<ConfigLoader>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            service_health: Arc::new(DashMap::new()),
            config_loader,
            client,
        }
    }

    /// Check if a service is healthy
    pub fn is_healthy(&self, service_id: &Uuid) -> bool {
        match self.service_health.get(service_id) {
            Some(status) => *status == HealthStatus::Healthy,
            None => true, // Default to healthy if not checked yet
        }
    }

    /// Get health status for a service
    pub fn get_status(&self, service_id: &Uuid) -> HealthStatus {
        self.service_health
            .get(service_id)
            .map(|s| *s)
            .unwrap_or(HealthStatus::Unknown)
    }

    /// Start the health check background task
    pub fn start_background_checker(self: Arc<Self>) {
        tokio::spawn(async move {
            info!("Starting health check background task");
            let mut check_interval = interval(Duration::from_secs(10));

            loop {
                check_interval.tick().await;
                self.check_all_services().await;
            }
        });
    }

    /// Check health for all services
    async fn check_all_services(&self) {
        let config = self.config_loader.get_config();

        for (service_id, service) in &config.services {
            // Only check services that have a health_check_url configured
            if service.health_check_url.is_some() {
                self.check_service(*service_id, service).await;
            }
        }
    }

    /// Check health for a single service
    async fn check_service(&self, service_id: Uuid, service: &BackendService) {
        let health_url = match &service.health_check_url {
            Some(url) => url,
            None => return, // Skip if no health check URL
        };

        // Build full health check URL
        let full_url = if health_url.starts_with("http://") || health_url.starts_with("https://") {
            health_url.clone()
        } else {
            format!("{}{}", service.base_url, health_url)
        };

        debug!(
            "Checking health for service {} ({}): {}",
            service.name, service_id, full_url
        );

        // Perform health check
        let is_healthy = match self.client.get(&full_url).send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    debug!("Service {} is healthy (status: {})", service.name, status);
                    true
                } else {
                    warn!(
                        "Service {} returned non-success status: {}",
                        service.name, status
                    );
                    false
                }
            }
            Err(e) => {
                error!("Health check failed for service {}: {}", service.name, e);
                false
            }
        };

        // Update health status
        let new_status = if is_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };

        // Log status changes
        let old_status = self.service_health.get(&service_id).map(|s| *s);
        if old_status != Some(new_status) {
            info!(
                "Service {} ({}) status changed: {:?} -> {:?}",
                service.name, service_id, old_status, new_status
            );
        }

        self.service_health.insert(service_id, new_status);
    }

    /// Get all service health statuses
    pub fn get_all_statuses(&self) -> Vec<(Uuid, HealthStatus)> {
        self.service_health
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect()
    }
}
