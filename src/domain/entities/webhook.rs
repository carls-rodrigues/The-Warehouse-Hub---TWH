use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookEventType {
    StockMovement,
    PurchaseOrderCreated,
    PurchaseOrderUpdated,
    SalesOrderCreated,
    SalesOrderUpdated,
    TransferCreated,
    TransferUpdated,
    ReturnCreated,
    ReturnUpdated,
    AdjustmentCreated,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookEventType::StockMovement => "STOCK_MOVEMENT",
            WebhookEventType::PurchaseOrderCreated => "PURCHASE_ORDER_CREATED",
            WebhookEventType::PurchaseOrderUpdated => "PURCHASE_ORDER_UPDATED",
            WebhookEventType::SalesOrderCreated => "SALES_ORDER_CREATED",
            WebhookEventType::SalesOrderUpdated => "SALES_ORDER_UPDATED",
            WebhookEventType::TransferCreated => "TRANSFER_CREATED",
            WebhookEventType::TransferUpdated => "TRANSFER_UPDATED",
            WebhookEventType::ReturnCreated => "RETURN_CREATED",
            WebhookEventType::ReturnUpdated => "RETURN_UPDATED",
            WebhookEventType::AdjustmentCreated => "ADJUSTMENT_CREATED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_uppercase().as_str() {
            "STOCK_MOVEMENT" => Ok(WebhookEventType::StockMovement),
            "PURCHASE_ORDER_CREATED" => Ok(WebhookEventType::PurchaseOrderCreated),
            "PURCHASE_ORDER_UPDATED" => Ok(WebhookEventType::PurchaseOrderUpdated),
            "SALES_ORDER_CREATED" => Ok(WebhookEventType::SalesOrderCreated),
            "SALES_ORDER_UPDATED" => Ok(WebhookEventType::SalesOrderUpdated),
            "TRANSFER_CREATED" => Ok(WebhookEventType::TransferCreated),
            "TRANSFER_UPDATED" => Ok(WebhookEventType::TransferUpdated),
            "RETURN_CREATED" => Ok(WebhookEventType::ReturnCreated),
            "RETURN_UPDATED" => Ok(WebhookEventType::ReturnUpdated),
            "ADJUSTMENT_CREATED" => Ok(WebhookEventType::AdjustmentCreated),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid webhook event type: {}. Must be one of: STOCK_MOVEMENT, PURCHASE_ORDER_CREATED, PURCHASE_ORDER_UPDATED, SALES_ORDER_CREATED, SALES_ORDER_UPDATED, TRANSFER_CREATED, TRANSFER_UPDATED, RETURN_CREATED, RETURN_UPDATED, ADJUSTMENT_CREATED",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookStatus {
    Active,
    Inactive,
    Failed,
}

impl WebhookStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookStatus::Active => "ACTIVE",
            WebhookStatus::Inactive => "INACTIVE",
            WebhookStatus::Failed => "FAILED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Ok(WebhookStatus::Active),
            "INACTIVE" => Ok(WebhookStatus::Inactive),
            "FAILED" => Ok(WebhookStatus::Failed),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid webhook status: {}. Must be one of: ACTIVE, INACTIVE, FAILED",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeliveryStatus {
    Pending,
    Success,
    Failed,
    Timeout,
    Dlq,
}

impl DeliveryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryStatus::Pending => "PENDING",
            DeliveryStatus::Success => "SUCCESS",
            DeliveryStatus::Failed => "FAILED",
            DeliveryStatus::Timeout => "TIMEOUT",
            DeliveryStatus::Dlq => "DLQ",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s.to_uppercase().as_str() {
            "PENDING" => Ok(DeliveryStatus::Pending),
            "SUCCESS" => Ok(DeliveryStatus::Success),
            "FAILED" => Ok(DeliveryStatus::Failed),
            "TIMEOUT" => Ok(DeliveryStatus::Timeout),
            "DLQ" => Ok(DeliveryStatus::Dlq),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid delivery status: {}. Must be one of: PENDING, SUCCESS, FAILED, TIMEOUT, DLQ",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<WebhookEventType>,
    pub status: WebhookStatus,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_delivery_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

impl Webhook {
    pub fn new(
        url: String,
        secret: String,
        events: Vec<WebhookEventType>,
        created_by: Uuid,
    ) -> Result<Self, DomainError> {
        // Validate URL format
        if url.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Webhook URL cannot be empty".to_string(),
            ));
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(DomainError::ValidationError(
                "Webhook URL must start with http:// or https://".to_string(),
            ));
        }

        // Validate secret
        if secret.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Webhook secret cannot be empty".to_string(),
            ));
        }

        // Validate events
        if events.is_empty() {
            return Err(DomainError::ValidationError(
                "Webhook must subscribe to at least one event".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            url,
            secret,
            events,
            status: WebhookStatus::Active,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_delivery_at: None,
            failure_count: 0,
        })
    }

    pub fn update_status(&mut self, status: WebhookStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn record_delivery_attempt(&mut self, success: bool) {
        self.last_delivery_at = Some(Utc::now());
        if success {
            self.failure_count = 0;
        } else {
            self.failure_count += 1;
        }
        self.updated_at = Utc::now();

        // Auto-disable webhook after too many failures
        if self.failure_count >= 10 {
            self.status = WebhookStatus::Failed;
        }
    }

    pub fn is_subscribed_to(&self, event_type: &WebhookEventType) -> bool {
        self.events.contains(event_type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub event_type: WebhookEventType,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

impl WebhookEvent {
    pub fn new(event_type: WebhookEventType, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            payload,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_id: Uuid,
    pub status: DeliveryStatus,
    pub attempt_count: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WebhookDelivery {
    pub fn new(webhook_id: Uuid, event_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            webhook_id,
            event_id,
            status: DeliveryStatus::Pending,
            attempt_count: 0,
            last_attempt_at: None,
            next_attempt_at: Some(Utc::now()),
            response_status: None,
            response_body: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn record_attempt(
        &mut self,
        success: bool,
        response_status: Option<i32>,
        response_body: Option<String>,
        error_message: Option<String>,
    ) {
        self.attempt_count += 1;
        self.last_attempt_at = Some(Utc::now());
        self.response_status = response_status;
        self.response_body = response_body;
        self.error_message = error_message;
        self.updated_at = Utc::now();

        if success {
            self.status = DeliveryStatus::Success;
            self.next_attempt_at = None;
        } else {
            // Exponential backoff: 1min, 5min, 30min, 2h, 8h, then DLQ
            let next_delay = match self.attempt_count {
                1 => chrono::Duration::minutes(1),
                2 => chrono::Duration::minutes(5),
                3 => chrono::Duration::minutes(30),
                4 => chrono::Duration::hours(2),
                5 => chrono::Duration::hours(8),
                _ => {
                    self.status = DeliveryStatus::Dlq;
                    return;
                }
            };

            self.next_attempt_at = Some(Utc::now() + next_delay);
            self.status = DeliveryStatus::Failed;
        }
    }

    pub fn should_retry(&self) -> bool {
        matches!(self.status, DeliveryStatus::Failed) && self.attempt_count < 5
    }

    pub fn is_in_dlq(&self) -> bool {
        matches!(self.status, DeliveryStatus::Dlq)
    }
}

// Request/Response DTOs for API

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWebhookRequest {
    pub url: Option<String>,
    pub secret: Option<String>,
    pub events: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_delivery_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub webhook_id: Uuid,
    pub event_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub response_status: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookEventResponse {
    pub id: Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
