use karateway_core::{
    models::{CreateWhitelistRuleRequest, UpdateWhitelistRuleRequest, WhitelistRule},
    KaratewayError, Result,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct WhitelistRuleRepository {
    pool: PgPool,
}

impl WhitelistRuleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: CreateWhitelistRuleRequest) -> Result<WhitelistRule> {
        let rule = sqlx::query_as::<_, WhitelistRule>(
            r#"
            INSERT INTO whitelist_rules (
                rule_name, rule_type, api_route_id, config, priority
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&req.rule_name)
        .bind(&req.rule_type)
        .bind(req.api_route_id)
        .bind(&req.config)
        .bind(req.priority.unwrap_or(0))
        .fetch_one(&self.pool)
        .await?;

        Ok(rule)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<WhitelistRule> {
        let rule = sqlx::query_as::<_, WhitelistRule>(
            r#"
            SELECT * FROM whitelist_rules WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            KaratewayError::NotFound(format!("Whitelist rule with id {} not found", id))
        })?;

        Ok(rule)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<WhitelistRule>> {
        let offset = (page.saturating_sub(1)) * limit;

        let rules = sqlx::query_as::<_, WhitelistRule>(
            r#"
            SELECT * FROM whitelist_rules
            ORDER BY priority DESC, created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    pub async fn count(&self) -> Result<u64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM whitelist_rules
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_route(&self, api_route_id: Uuid) -> Result<Vec<WhitelistRule>> {
        let rules = sqlx::query_as::<_, WhitelistRule>(
            r#"
            SELECT * FROM whitelist_rules
            WHERE api_route_id = $1 AND is_active = true
            ORDER BY priority DESC
            "#,
        )
        .bind(api_route_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    pub async fn update(&self, id: Uuid, req: UpdateWhitelistRuleRequest) -> Result<WhitelistRule> {
        let mut rule = self.find_by_id(id).await?;

        if let Some(rule_name) = req.rule_name {
            rule.rule_name = rule_name;
        }
        if let Some(rule_type) = req.rule_type {
            rule.rule_type = rule_type;
        }
        if let Some(api_route_id) = req.api_route_id {
            rule.api_route_id = Some(api_route_id);
        }
        if let Some(config) = req.config {
            rule.config = config;
        }
        if let Some(is_active) = req.is_active {
            rule.is_active = is_active;
        }
        if let Some(priority) = req.priority {
            rule.priority = priority;
        }

        let updated = sqlx::query_as::<_, WhitelistRule>(
            r#"
            UPDATE whitelist_rules
            SET rule_name = $1, rule_type = $2, api_route_id = $3,
                config = $4, is_active = $5, priority = $6, updated_at = CURRENT_TIMESTAMP
            WHERE id = $7
            RETURNING *
            "#,
        )
        .bind(&rule.rule_name)
        .bind(&rule.rule_type)
        .bind(rule.api_route_id)
        .bind(&rule.config)
        .bind(rule.is_active)
        .bind(rule.priority)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM whitelist_rules WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(KaratewayError::NotFound(format!(
                "Whitelist rule with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn list_active(&self) -> Result<Vec<WhitelistRule>> {
        let rules = sqlx::query_as::<_, WhitelistRule>(
            r#"
            SELECT * FROM whitelist_rules
            WHERE is_active = true
            ORDER BY priority DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }
}
