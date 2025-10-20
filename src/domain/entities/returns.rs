use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReturnStatus {
    Draft,
    Open,
    Received,
    Cancelled,
}

impl ReturnStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReturnStatus::Draft => "DRAFT",
            ReturnStatus::Open => "OPEN",
            ReturnStatus::Received => "RECEIVED",
            ReturnStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "DRAFT" => Ok(ReturnStatus::Draft),
            "OPEN" => Ok(ReturnStatus::Open),
            "RECEIVED" => Ok(ReturnStatus::Received),
            "CANCELLED" => Ok(ReturnStatus::Cancelled),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid return status: {}",
                s
            ))),
        }
    }

    pub fn can_transition_to(&self, new_status: &ReturnStatus) -> bool {
        match self {
            ReturnStatus::Draft => {
                matches!(new_status, ReturnStatus::Open | ReturnStatus::Cancelled)
            }
            ReturnStatus::Open => {
                matches!(new_status, ReturnStatus::Received | ReturnStatus::Cancelled)
            }
            ReturnStatus::Received => false,
            ReturnStatus::Cancelled => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Return {
    pub id: Uuid,
    pub return_number: String,
    pub customer_id: Option<Uuid>,
    pub location_id: Uuid,
    pub status: ReturnStatus,
    pub total_quantity: i32,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lines: Vec<ReturnLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnLine {
    pub id: Uuid,
    pub return_id: Uuid,
    pub item_id: Uuid,
    pub quantity: i32,
    pub quantity_received: i32,
    pub unit_price: f64,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnRequest {
    pub customer_id: Option<Uuid>,
    pub location_id: Uuid,
    pub lines: Vec<CreateReturnLineRequest>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnLineRequest {
    pub item_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReturnRequest {
    pub lines: Vec<ProcessReturnLineRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessReturnLineRequest {
    pub return_line_id: Uuid,
    pub quantity_received: i32,
}

impl Return {
    pub fn new(
        return_number: String,
        customer_id: Option<Uuid>,
        location_id: Uuid,
        created_by: Uuid,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            id: Uuid::new_v4(),
            return_number,
            customer_id,
            location_id,
            status: ReturnStatus::Draft,
            total_quantity: 0,
            notes: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            lines: Vec::new(),
        })
    }

    pub fn add_line(&mut self, line: ReturnLine) -> Result<(), DomainError> {
        if line.quantity <= 0 {
            return Err(DomainError::ValidationError(
                "Return line quantity must be positive".to_string(),
            ));
        }

        if line.unit_price < 0.0 {
            return Err(DomainError::ValidationError(
                "Return line unit price cannot be negative".to_string(),
            ));
        }

        self.lines.push(line);
        self.total_quantity = self.lines.iter().map(|l| l.quantity).sum();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn open(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&ReturnStatus::Open) {
            return Err(DomainError::ValidationError(format!(
                "Cannot open return with status: {:?}",
                self.status
            )));
        }

        self.status = ReturnStatus::Open;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn process(
        &mut self,
        processed_lines: Vec<ProcessReturnLineRequest>,
    ) -> Result<Vec<crate::domain::entities::inventory::StockMovement>, DomainError> {
        if self.status != ReturnStatus::Open {
            return Err(DomainError::ValidationError(format!(
                "Cannot process return with status: {:?}",
                self.status
            )));
        }

        if processed_lines.is_empty() {
            return Err(DomainError::ValidationError(
                "No lines specified for processing".to_string(),
            ));
        }

        let mut stock_movements = Vec::new();

        for process_request in processed_lines {
            let line = self
                .lines
                .iter_mut()
                .find(|l| l.id == process_request.return_line_id)
                .ok_or_else(|| {
                    DomainError::ValidationError(format!(
                        "Line {} not found",
                        process_request.return_line_id
                    ))
                })?;

            if process_request.quantity_received > line.quantity {
                return Err(DomainError::ValidationError(format!(
                    "Cannot receive {} units of line {}, only {} returned",
                    process_request.quantity_received, line.id, line.quantity
                )));
            }

            if process_request.quantity_received <= 0 {
                return Err(DomainError::ValidationError(
                    "Received quantity must be positive".to_string(),
                ));
            }

            line.quantity_received = process_request.quantity_received;
            line.updated_at = Utc::now();

            // Create inbound movement to location (items coming back into inventory)
            let movement = crate::domain::entities::inventory::StockMovement::new(
                line.item_id,
                self.location_id,
                crate::domain::entities::inventory::MovementType::Inbound,
                process_request.quantity_received,
                crate::domain::entities::inventory::ReferenceType::Return,
                Some(self.id),
                Some(format!(
                    "Return inbound: {} units of item {}",
                    process_request.quantity_received, line.item_id
                )),
                Some(self.created_by),
            )?;
        }

        // Check if all lines are fully received
        let total_received: i32 = self.lines.iter().map(|l| l.quantity_received).sum();
        let total_returned: i32 = self.lines.iter().map(|l| l.quantity).sum();

        if total_received >= total_returned {
            self.status = ReturnStatus::Received;
        }

        self.updated_at = Utc::now();
        Ok(stock_movements)
    }
}

impl ReturnLine {
    pub fn new(
        return_id: Uuid,
        item_id: Uuid,
        quantity: i32,
        unit_price: f64,
        reason: Option<String>,
    ) -> Result<Self, DomainError> {
        if quantity <= 0 {
            return Err(DomainError::ValidationError(
                "Return line quantity must be positive".to_string(),
            ));
        }

        if unit_price < 0.0 {
            return Err(DomainError::ValidationError(
                "Return line unit price cannot be negative".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            return_id,
            item_id,
            quantity,
            quantity_received: 0,
            unit_price,
            reason,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
