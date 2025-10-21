use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::sync::Arc;

use crate::domain::entities::tenant::TenantTier;
use crate::infrastructure::middleware::tenant_middleware::TenantContext;
use crate::infrastructure::observability::metrics::AppMetrics;

#[derive(Clone)]
pub struct RateLimitMiddleware {
    redis_client: redis::Client,
}

impl RateLimitMiddleware {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            redis_client: client,
        })
    }

    pub async fn handle(self, headers: HeaderMap, request: Request, next: Next) -> Response {
        // Extract tenant context from request extensions
        let tenant_context = request.extensions().get::<TenantContext>();

        let tenant_tier = match tenant_context {
            Some(ctx) => ctx.tier.clone(),
            None => {
                // No tenant context found, default to FREE tier
                TenantTier::Free
            }
        };

        // Get the endpoint path
        let path = request.uri().path();

        // Check rate limit
        match self.check_rate_limit(&tenant_tier, path).await {
            Ok((true, remaining, reset_time)) => {
                // Rate limit passed, continue with request
                let mut response = next.run(request).await;
                // Add rate limit headers
                response.headers_mut().insert(
                    "X-RateLimit-Remaining",
                    remaining.to_string().parse().unwrap(),
                );
                response
                    .headers_mut()
                    .insert("X-RateLimit-Reset", reset_time.to_string().parse().unwrap());
                response
            }
            Ok((false, remaining, reset_time)) => {
                // Rate limit exceeded
                let mut response = (
                    StatusCode::TOO_MANY_REQUESTS,
                    "Rate limit exceeded. Please try again later.",
                )
                    .into_response();
                response.headers_mut().insert(
                    "Retry-After",
                    "60".parse().unwrap(), // Retry after 60 seconds
                );
                response.headers_mut().insert(
                    "X-RateLimit-Remaining",
                    remaining.to_string().parse().unwrap(),
                );
                response
                    .headers_mut()
                    .insert("X-RateLimit-Reset", reset_time.to_string().parse().unwrap());
                response
            }
            Err(_) => {
                // Redis error, allow request to proceed (fail open)
                next.run(request).await
            }
        }
    }

    async fn check_rate_limit(
        &self,
        tenant_tier: &TenantTier,
        endpoint: &str,
    ) -> Result<(bool, i64, i64), redis::RedisError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        // Create a key for this tenant tier and endpoint
        let key = format!("ratelimit:{:?}:{}", tenant_tier, endpoint);

        // Use Redis sorted set to track requests in a sliding window
        let now = chrono::Utc::now().timestamp();
        let window_start = now - 60; // 60 second window

        // Remove old entries outside the window
        let _: () = conn
            .zrembyscore(&key, "-inf", &window_start.to_string())
            .await?;

        // Count current requests in the window
        let current_count: i64 = conn.zcount(&key, "-inf", "+inf").await?;

        // Check if limit exceeded based on tenant tier
        let limit = tenant_tier.requests_per_minute() as i64;
        let remaining = limit - current_count;
        let reset_time = now + 60; // Reset in 60 seconds

        if current_count >= limit {
            return Ok((false, remaining, reset_time));
        }

        // Add current request with unique member
        let request_id = format!("{}:{}", now, uuid::Uuid::new_v4().simple());
        let _: () = conn.zadd(&key, &request_id, now).await?;

        // Set expiration on the key (cleanup after window + buffer)
        let _: () = conn.expire(&key, 120).await?; // 2 minutes

        Ok((true, remaining - 1, reset_time)) // remaining - 1 because we just added one
    }
}

pub async fn rate_limit_middleware(
    state: axum::extract::State<Arc<RateLimitMiddleware>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Extract tenant context from request extensions
    let tenant_context = request.extensions().get::<TenantContext>();

    let tenant_tier = match tenant_context {
        Some(ctx) => ctx.tier.clone(),
        None => {
            // No tenant context found, default to FREE tier
            TenantTier::Free
        }
    };

    // Get the endpoint path
    let path = request.uri().path();

    // Check rate limit
    match state.check_rate_limit(&tenant_tier, path).await {
        Ok((true, remaining, reset_time)) => {
            // Rate limit passed, continue with request
            let mut response = next.run(request).await;
            // Add rate limit headers
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                remaining.to_string().parse().unwrap(),
            );
            response
                .headers_mut()
                .insert("X-RateLimit-Reset", reset_time.to_string().parse().unwrap());
            response
        }
        Ok((false, remaining, reset_time)) => {
            // Rate limit exceeded - record metrics
            if let Some(ctx) = tenant_context {
                AppMetrics::get()
                    .record_rate_limit_hit(&ctx.tenant_id.to_string(), &format!("{:?}", ctx.tier));
            }

            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded. Please try again later.",
            )
                .into_response();
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap(), // Retry after 60 seconds
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                remaining.to_string().parse().unwrap(),
            );
            response
                .headers_mut()
                .insert("X-RateLimit-Reset", reset_time.to_string().parse().unwrap());
            response
        }
        Err(_) => {
            // Redis error, allow request to proceed (fail open)
            next.run(request).await
        }
    }
}
