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
use uuid::Uuid;

use crate::domain::entities::tenant::TenantTier;
use crate::domain::services::tenant_repository::TenantRepository;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tier: TenantTier,
}

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
    tenant_repository: Arc<dyn TenantRepository>,
}

impl TenantMiddleware {
    pub fn new(
        pool: Arc<PgPool>,
        jwt_secret: String,
        tenant_repository: Arc<dyn TenantRepository>,
    ) -> Self {
        Self {
            pool,
            jwt_secret,
            tenant_repository,
        }
    }

    pub async fn handle(&self, headers: HeaderMap, mut request: Request, next: Next) -> Response {
        // Extract tenant_id from JWT token or X-Tenant-ID header
        let tenant_id = match self.extract_tenant_from_token(&headers).await {
            Ok(Some(tenant_id)) => Some(tenant_id),
            _ => {
                // Check for X-Tenant-ID header (for testing/development)
                if let Some(tenant_header) = headers.get("x-tenant-id") {
                    if let Ok(tenant_str) = tenant_header.to_str() {
                        uuid::Uuid::parse_str(tenant_str).ok()
                    } else {
                        None
                    }
                } else {
                    // Default tenant for development
                    Some(uuid::Uuid::parse_str("d60a7de9-1009-4606-aae9-ae6ffe5827aa").unwrap())
                }
            }
        };

        if let Some(tenant_id) = tenant_id {
            // Set tenant context in database session
            if let Err(_) = sqlx::query("SELECT set_tenant_context($1)")
                .bind(tenant_id)
                .execute(&*self.pool)
                .await
            {
                // If setting tenant context fails, return error
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to set tenant context",
                )
                    .into_response();
            }

            // Look up tenant tier from database
            let tier = match self.tenant_repository.get_tenant_tier(tenant_id).await {
                Ok(Some(tier)) => tier,
                _ => {
                    // If tenant not found or error, default to FREE tier
                    TenantTier::Free
                }
            };

            let tenant_context = TenantContext { tenant_id, tier };

            // Store tenant context in request extensions for use by other middleware and handlers
            request.extensions_mut().insert(tenant_context);
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
