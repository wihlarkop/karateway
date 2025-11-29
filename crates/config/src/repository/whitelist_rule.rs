use karateway_core::{
    models::{CreateWhitelistRuleRequest, UpdateWhitelistRuleRequest, WhitelistRule, WhitelistRules},
    KaratewayError, Result,
};
use sea_query::{Expr, Func, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
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
        let (sql, values) = Query::insert()
            .into_table(WhitelistRules::Table)
            .columns([
                WhitelistRules::RuleName,
                WhitelistRules::RuleType,
                WhitelistRules::ApiRouteId,
                WhitelistRules::Config,
                WhitelistRules::Priority,
            ])
            .values_panic([
                req.rule_name.into(),
                req.rule_type.to_string().into(),
                req.api_route_id.into(),
                req.config.into(),
                req.priority.unwrap_or(0).into(),
            ])
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let rule = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(rule)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<WhitelistRule> {
        let (sql, values) = Query::select()
            .columns([
                WhitelistRules::Id,
                WhitelistRules::RuleName,
                WhitelistRules::RuleType,
                WhitelistRules::ApiRouteId,
                WhitelistRules::Config,
                WhitelistRules::IsActive,
                WhitelistRules::Priority,
                WhitelistRules::CreatedAt,
                WhitelistRules::UpdatedAt,
            ])
            .from(WhitelistRules::Table)
            .and_where(Expr::col(WhitelistRules::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let rule = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| {
                KaratewayError::NotFound(format!("Whitelist rule with id {} not found", id))
            })?;

        Ok(rule)
    }

    pub async fn list(&self, page: u32, limit: u32) -> Result<Vec<WhitelistRule>> {
        let offset = (page.saturating_sub(1)) * limit;

        let (sql, values) = Query::select()
            .columns([
                WhitelistRules::Id,
                WhitelistRules::RuleName,
                WhitelistRules::RuleType,
                WhitelistRules::ApiRouteId,
                WhitelistRules::Config,
                WhitelistRules::IsActive,
                WhitelistRules::Priority,
                WhitelistRules::CreatedAt,
                WhitelistRules::UpdatedAt,
            ])
            .from(WhitelistRules::Table)
            .order_by(WhitelistRules::Priority, sea_query::Order::Desc)
            .order_by(WhitelistRules::CreatedAt, sea_query::Order::Desc)
            .limit(limit as u64)
            .offset(offset as u64)
            .build_sqlx(PostgresQueryBuilder);

        let rules = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rules)
    }

    pub async fn count(&self) -> Result<u64> {
        let (sql, values) = Query::select()
            .expr(Func::count(Expr::col(WhitelistRules::Id)))
            .from(WhitelistRules::Table)
            .build_sqlx(PostgresQueryBuilder);

        let count: (i64,) = sqlx::query_as_with(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 as u64)
    }

    pub async fn list_by_route(&self, api_route_id: Uuid) -> Result<Vec<WhitelistRule>> {
        let (sql, values) = Query::select()
            .columns([
                WhitelistRules::Id,
                WhitelistRules::RuleName,
                WhitelistRules::RuleType,
                WhitelistRules::ApiRouteId,
                WhitelistRules::Config,
                WhitelistRules::IsActive,
                WhitelistRules::Priority,
                WhitelistRules::CreatedAt,
                WhitelistRules::UpdatedAt,
            ])
            .from(WhitelistRules::Table)
            .and_where(Expr::col(WhitelistRules::ApiRouteId).eq(api_route_id))
            .and_where(Expr::col(WhitelistRules::IsActive).eq(true))
            .order_by(WhitelistRules::Priority, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let rules = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
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

        let (sql, values) = Query::update()
            .table(WhitelistRules::Table)
            .values([
                (WhitelistRules::RuleName, rule.rule_name.clone().into()),
                (WhitelistRules::RuleType, rule.rule_type.to_string().into()),
                (WhitelistRules::ApiRouteId, rule.api_route_id.into()),
                (WhitelistRules::Config, rule.config.clone().into()),
                (WhitelistRules::IsActive, rule.is_active.into()),
                (WhitelistRules::Priority, rule.priority.into()),
            ])
            .and_where(Expr::col(WhitelistRules::Id).eq(id))
            .returning_all()
            .build_sqlx(PostgresQueryBuilder);

        let updated = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
            .fetch_one(&self.pool)
            .await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let (sql, values) = Query::delete()
            .from_table(WhitelistRules::Table)
            .and_where(Expr::col(WhitelistRules::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        let result = sqlx::query_with(&sql, values)
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
        let (sql, values) = Query::select()
            .columns([
                WhitelistRules::Id,
                WhitelistRules::RuleName,
                WhitelistRules::RuleType,
                WhitelistRules::ApiRouteId,
                WhitelistRules::Config,
                WhitelistRules::IsActive,
                WhitelistRules::Priority,
                WhitelistRules::CreatedAt,
                WhitelistRules::UpdatedAt,
            ])
            .from(WhitelistRules::Table)
            .and_where(Expr::col(WhitelistRules::IsActive).eq(true))
            .order_by(WhitelistRules::Priority, sea_query::Order::Desc)
            .build_sqlx(PostgresQueryBuilder);

        let rules = sqlx::query_as_with::<_, WhitelistRule, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rules)
    }
}
