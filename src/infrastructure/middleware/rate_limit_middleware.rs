use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum TenantTier {
    Free,
    Developer,
    Startup,
    Growth,
    Scale,
    Enterprise,
}

impl TenantTier {
    pub fn requests_per_minute(&self) -> u32 {
        match self {
            TenantTier::Free => 10,      // 10k/month = ~10/minute (generous for sandbox)
            TenantTier::Developer => 50, // 100k/month = ~50/minute
            TenantTier::Startup => 200,  // 500k/month = ~200/minute
            TenantTier::Growth => 800,   // 2M/month = ~800/minute
            TenantTier::Scale => 4000,   // 10M/month = ~4000/minute
            TenantTier::Enterprise => 10000, // Custom high limit for enterprise
        }
    }
}

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
        // For now, use default tier. In production, this would come from tenant lookup
        let tenant_tier = TenantTier::Free; // Default to free tier

        // Get the endpoint path
        let path = request.uri().path();

        // Check rate limit
        match self.check_rate_limit(&tenant_tier, path).await {
            Ok(true) => {
                // Rate limit passed, continue with request
                next.run(request).await
            }
            Ok(false) => {
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
    ) -> Result<bool, redis::RedisError> {
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
        if current_count >= limit {
            return Ok(false);
        }

        // Add current request with unique member
        let request_id = format!("{}:{}", now, uuid::Uuid::new_v4().simple());
        let _: () = conn.zadd(&key, &request_id, now).await?;

        // Set expiration on the key (cleanup after window + buffer)
        let _: () = conn.expire(&key, 120).await?; // 2 minutes

        Ok(true)
    }
}

pub async fn rate_limit_middleware(
    state: axum::extract::State<Arc<RateLimitMiddleware>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // For now, use default tier. In production, this would come from tenant lookup
    let tenant_tier = TenantTier::Free; // Default to free tier

    // Get the endpoint path
    let path = request.uri().path();

    // Check rate limit
    match state.check_rate_limit(&tenant_tier, path).await {
        Ok(true) => {
            // Rate limit passed, continue with request
            next.run(request).await
        }
        Ok(false) => {
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
            response
        }
        Err(_) => {
            // Redis error, allow request to proceed (fail open)
            next.run(request).await
        }
    }
}
