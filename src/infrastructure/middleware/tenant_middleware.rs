use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    email: String,
    tenant_id: String,
    exp: usize,
    iat: usize,
}

#[derive(Clone)]
pub struct TenantMiddleware {
    pool: Arc<PgPool>,
    jwt_secret: String,
}

impl TenantMiddleware {
    pub fn new(pool: Arc<PgPool>, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn handle(self, headers: HeaderMap, mut request: Request, next: Next) -> Response {
        // Extract tenant_id from JWT token
        let tenant_id = match self.extract_tenant_from_token(&headers).await {
            Ok(tenant_id) => tenant_id,
            Err(_) => {
                // For now, allow requests without authentication to proceed with default tenant
                // In production, this should return 401 Unauthorized
                Some(uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap())
            }
        };

        if let Some(tenant_id) = tenant_id {
            // Set tenant context in the database connection
            // Note: In a real implementation, you'd want to use a connection pool that supports
            // per-request tenant context, or modify the repositories to accept tenant_id as a parameter.
            // For now, we'll set a session variable that RLS policies can use.

            // Since we can't easily modify the connection for each request with the current architecture,
            // we'll store the tenant_id in request extensions for the handlers to use
            request.extensions_mut().insert(tenant_id);
        }

        next.run(request).await
    }

    async fn extract_tenant_from_token(
        &self,
        headers: &HeaderMap,
    ) -> Result<Option<uuid::Uuid>, Box<dyn std::error::Error + Send + Sync>> {
        let auth_header = match headers.get(AUTHORIZATION) {
            Some(header) => header.to_str()?,
            None => return Ok(None),
        };

        if !auth_header.starts_with("Bearer ") {
            return Ok(None);
        }

        let token = &auth_header[7..]; // Remove "Bearer " prefix

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        let tenant_id = uuid::Uuid::parse_str(&token_data.claims.tenant_id)?;
        Ok(Some(tenant_id))
    }
}

pub async fn tenant_middleware(headers: HeaderMap, request: Request, next: Next) -> Response {
    // This is a placeholder - in a real implementation, you'd get the middleware from app state
    // For now, we'll skip tenant validation and just pass through
    next.run(request).await
}
