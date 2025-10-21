use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantType {
    Production,
    Sandbox,
}

impl TenantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TenantType::Production => "PRODUCTION",
            TenantType::Sandbox => "SANDBOX",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_uppercase().as_str() {
            "PRODUCTION" => Ok(TenantType::Production),
            "SANDBOX" => Ok(TenantType::Sandbox),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid tenant type: {}. Must be one of: PRODUCTION, SANDBOX",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantStatus {
    Provisioning,
    Active,
    Suspended,
    Deleting,
}

impl TenantStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TenantStatus::Provisioning => "PROVISIONING",
            TenantStatus::Active => "ACTIVE",
            TenantStatus::Suspended => "SUSPENDED",
            TenantStatus::Deleting => "DELETING",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_uppercase().as_str() {
            "PROVISIONING" => Ok(TenantStatus::Provisioning),
            "ACTIVE" => Ok(TenantStatus::Active),
            "SUSPENDED" => Ok(TenantStatus::Suspended),
            "DELETING" => Ok(TenantStatus::Deleting),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid tenant status: {}. Must be one of: PROVISIONING, ACTIVE, SUSPENDED, DELETING",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub tenant_type: TenantType,
    pub status: TenantStatus,
    pub database_schema: String,
    pub created_by: Option<Uuid>, // User who created the tenant (None for system-created)
    pub expires_at: Option<DateTime<Utc>>, // For sandbox tenants
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    pub fn new(
        name: String,
        tenant_type: TenantType,
        database_schema: String,
        created_by: Option<Uuid>,
    ) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Tenant name cannot be empty".to_string(),
            ));
        }

        if database_schema.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Database schema cannot be empty".to_string(),
            ));
        }

        let expires_at = match tenant_type {
            TenantType::Sandbox => Some(Utc::now() + chrono::Duration::days(30)),
            TenantType::Production => None,
        };

        let status = match tenant_type {
            TenantType::Sandbox => TenantStatus::Provisioning,
            TenantType::Production => TenantStatus::Active,
        };

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            tenant_type,
            status,
            database_schema,
            created_by,
            expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn new_sandbox(created_by: Option<Uuid>) -> Self {
        let id = Uuid::new_v4();
        let expires_at = Some(Utc::now() + chrono::Duration::days(30));

        Self {
            id,
            name: format!("sandbox-{}", id.simple()),
            tenant_type: TenantType::Sandbox,
            status: TenantStatus::Provisioning,
            database_schema: format!("tenant_{}", id.simple()),
            created_by,
            expires_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn mark_active(&mut self) {
        self.status = TenantStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn mark_deleting(&mut self) {
        self.status = TenantStatus::Deleting;
        self.updated_at = Utc::now();
    }
}

// Request/Response DTOs for API

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSandboxTenantRequest {
    // No fields needed for basic sandbox creation
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSandboxTenantResponse {
    pub tenant_id: Uuid,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantStatusResponse {
    pub tenant_id: Uuid,
    pub status: String,
    pub tenant_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub sample_data_loaded: bool,
}
