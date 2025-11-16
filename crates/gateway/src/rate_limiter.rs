use anyhow::Result;
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Rate limiter using Redis with sliding window algorithm
pub struct RateLimiter {
    redis_client: redis::Client,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis_client = redis::Client::open(redis_url)?;
        Ok(Self { redis_client })
    }

    /// Check if a request is allowed under rate limiting
    /// Returns (allowed, remaining, reset_time)
    pub async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: i32,
        window_seconds: i32,
    ) -> Result<(bool, i32, u64)> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let window_start = now - window_seconds as u64;
        let redis_key = format!("ratelimit:{}", key);

        // Use Redis sorted set with timestamps as scores
        // Remove old entries outside the window
        let _: () = conn.zrembyscore(&redis_key, 0, window_start as f64).await?;

        // Count current requests in window
        let count: i32 = conn.zcard(&redis_key).await?;

        debug!(
            "Rate limit check: key={}, count={}/{}, window={}s",
            key, count, max_requests, window_seconds
        );

        if count >= max_requests {
            // Rate limit exceeded
            let oldest: Option<(String, f64)> = conn.zrange_withscores(&redis_key, 0, 0).await?;
            let reset_time = if let Some((_, score)) = oldest {
                (score as u64) + window_seconds as u64
            } else {
                now + window_seconds as u64
            };

            warn!(
                "Rate limit exceeded: key={}, count={}/{}, reset_in={}s",
                key,
                count,
                max_requests,
                reset_time.saturating_sub(now)
            );

            Ok((false, 0, reset_time))
        } else {
            // Allow request and add to sorted set
            let request_id = format!("{}:{}", now, uuid::Uuid::new_v4());
            let _: () = conn.zadd(&redis_key, request_id, now as f64).await?;

            // Set expiry to a window and some buffer
            let _: () = conn
                .expire(&redis_key, (window_seconds + 60) as i64)
                .await?;

            let remaining = max_requests - count - 1;
            let reset_time = now + window_seconds as u64;

            debug!(
                "Rate limit allowed: key={}, remaining={}, reset_in={}s",
                key, remaining, window_seconds
            );

            Ok((true, remaining, reset_time))
        }
    }

    /// Check rate limit with token bucket algorithm (supports burst)
    pub async fn check_rate_limit_with_burst(
        &self,
        key: &str,
        max_requests: i32,
        window_seconds: i32,
        burst_size: i32,
    ) -> Result<(bool, i32, u64)> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let redis_key = format!("ratelimit:bucket:{}", key);

        // Get current token count and last refill time
        let (tokens, last_refill): (Option<i32>, Option<u64>) = redis::pipe()
            .hget(&redis_key, "tokens")
            .hget(&redis_key, "last_refill")
            .query_async(&mut conn)
            .await?;

        let refill_rate = max_requests as f64 / window_seconds as f64;
        let max_tokens = max_requests + burst_size;

        let (mut current_tokens, last_refill_time) = match (tokens, last_refill) {
            (Some(t), Some(l)) => (t, l),
            _ => (max_tokens, now),
        };

        // Refill tokens based on time elapsed
        let elapsed = now.saturating_sub(last_refill_time);
        let tokens_to_add = (elapsed as f64 * refill_rate) as i32;
        current_tokens = (current_tokens + tokens_to_add).min(max_tokens);

        if current_tokens > 0 {
            // Allow request and consume one token
            current_tokens -= 1;

            redis::pipe()
                .hset(&redis_key, "tokens", current_tokens)
                .hset(&redis_key, "last_refill", now)
                .expire(&redis_key, (window_seconds * 2) as i64)
                .query_async::<()>(&mut conn)
                .await?;

            let reset_time = now + ((max_tokens - current_tokens) as f64 / refill_rate) as u64;

            debug!(
                "Rate limit allowed (token bucket): key={}, tokens_remaining={}, reset_in={}s",
                key,
                current_tokens,
                reset_time.saturating_sub(now)
            );

            Ok((true, current_tokens, reset_time))
        } else {
            // Rate limit exceeded
            let reset_time = now + (1.0 / refill_rate) as u64;

            warn!(
                "Rate limit exceeded (token bucket): key={}, no tokens available",
                key
            );

            Ok((false, 0, reset_time))
        }
    }
}
