use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};

use crate::domain::entities::idempotency::{IdempotencyKey, IdempotencyKeyRequest};
use crate::domain::services::idempotency_repository::IdempotencyRepository;
use crate::shared::error::DomainError;

pub async fn idempotency_middleware<R: IdempotencyRepository + 'static>(
    idempotency_repo: Arc<R>,
    request: Request,
    next: Next,
) -> Response {
    // Only apply to POST, PUT, PATCH methods
    if !matches!(
        request.method(),
        &Method::POST | &Method::PUT | &Method::PATCH
    ) {
        return next.run(request).await;
    }

    // Extract idempotency key from header
    let idempotency_key = match extract_idempotency_key(&request.headers()) {
        Some(key) => key,
        None => {
            // No idempotency key provided, proceed normally
            return next.run(request).await;
        }
    };

    // Check if key already exists
    match idempotency_repo.get_key(&idempotency_key).await {
        Ok(Some(existing_key)) => {
            // Key exists, return cached response
            return create_response_from_key(&existing_key);
        }
        Ok(None) => {
            // Key doesn't exist, proceed with request
        }
        Err(DomainError::InfrastructureError(_)) => {
            // Infrastructure error (Redis/PostgreSQL down), but we should still allow the request
            // In production, you might want to add circuit breaker logic here
            return next.run(request).await;
        }
        Err(e) => {
            // Other errors, return error response
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }

    // Clone request for processing
    let (parts, body) = request.into_parts();

    // Calculate request body hash
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Invalid request body").into_response();
        }
    };

    let body_hash = calculate_body_hash(&body_bytes);

    // Create idempotency key record
    let key_request = IdempotencyKeyRequest {
        idempotency_key: idempotency_key.clone(),
        request_path: parts.uri.path().to_string(),
        request_method: parts.method.to_string(),
        request_body_hash: body_hash,
        ttl_seconds: Some(86400), // 24 hours
    };

    let key = match IdempotencyKey::new(key_request) {
        Ok(key) => key,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, e.to_string()).into_response();
        }
    };

    // Store the key
    if let Err(e) = idempotency_repo.store_key(&key).await {
        match e {
            DomainError::Conflict(_) => {
                // Key was created by another concurrent request, try to get it
                match idempotency_repo.get_key(&idempotency_key).await {
                    Ok(Some(existing_key)) => {
                        return create_response_from_key(&existing_key);
                    }
                    _ => {
                        // Still can't get it, return conflict
                        return (StatusCode::CONFLICT, "Request already in progress")
                            .into_response();
                    }
                }
            }
            DomainError::InfrastructureError(_) => {
                // Infrastructure error, proceed without idempotency
                return next
                    .run(Request::from_parts(parts, Body::from(body_bytes)))
                    .await;
            }
            _ => {
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        }
    }

    // Reconstruct request and proceed
    let request = Request::from_parts(parts, Body::from(body_bytes));
    let response = next.run(request).await;

    // Store the response in the background
    let idempotency_repo_clone = Arc::clone(&idempotency_repo);
    let idempotency_key_clone = idempotency_key.clone();
    let status = response.status().as_u16() as i32;
    let body_string = if let Some(body) = extract_response_body(&response) {
        Some(body)
    } else {
        None
    };

    tokio::spawn(async move {
        let _ = idempotency_repo_clone
            .complete_key(&idempotency_key_clone, status, body_string)
            .await;
    });

    response
}

fn extract_idempotency_key(headers: &HeaderMap) -> Option<String> {
    headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

fn calculate_body_hash(body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    format!("{:x}", hasher.finalize())
}

fn create_response_from_key(key: &IdempotencyKey) -> Response {
    let status = match key.response_status {
        Some(status) => {
            StatusCode::from_u16(status as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        }
        None => StatusCode::ACCEPTED, // Request in progress
    };

    let body = key.response_body.clone().unwrap_or_default();

    (status, body).into_response()
}

fn extract_response_body(response: &Response) -> Option<String> {
    // This is a simplified version. In a real implementation, you'd need to
    // handle different response body types properly
    // For now, we'll just return None to avoid complexity
    None
}
