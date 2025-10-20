use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::item::Item;
use crate::domain::entities::location::Location;
use crate::domain::entities::user::User;
use crate::shared::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MovementType {
    Inbound,
    Outbound,
    Adjustment,
    Transfer,
    Initial,
}

impl MovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MovementType::Inbound => "inbound",
            MovementType::Outbound => "outbound",
            MovementType::Adjustment => "adjustment",
            MovementType::Transfer => "transfer",
            MovementType::Initial => "initial",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "inbound" => Ok(MovementType::Inbound),
            "outbound" => Ok(MovementType::Outbound),
            "adjustment" => Ok(MovementType::Adjustment),
            "transfer" => Ok(MovementType::Transfer),
            "initial" => Ok(MovementType::Initial),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid movement type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    PurchaseOrder,
    SalesOrder,
    Adjustment,
    Transfer,
    Return,
    Initial,
}

impl ReferenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReferenceType::PurchaseOrder => "purchase_order",
            ReferenceType::SalesOrder => "sales_order",
            ReferenceType::Adjustment => "adjustment",
            ReferenceType::Transfer => "transfer",
            ReferenceType::Return => "return",
            ReferenceType::Initial => "initial",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "purchase_order" => Ok(ReferenceType::PurchaseOrder),
            "sales_order" => Ok(ReferenceType::SalesOrder),
            "adjustment" => Ok(ReferenceType::Adjustment),
            "transfer" => Ok(ReferenceType::Transfer),
            "return" => Ok(ReferenceType::Return),
            "initial" => Ok(ReferenceType::Initial),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid reference type: {}",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockMovement {
    pub id: Uuid,
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub movement_type: MovementType,
    pub quantity: i32,
    pub reference_type: ReferenceType,
    pub reference_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

impl StockMovement {
    pub fn new(
        item_id: Uuid,
        location_id: Uuid,
        movement_type: MovementType,
        quantity: i32,
        reference_type: ReferenceType,
        reference_id: Option<Uuid>,
        reason: Option<String>,
        created_by: Option<Uuid>,
    ) -> Result<Self, DomainError> {
        // Validate quantity based on movement type
        match movement_type {
            MovementType::Inbound | MovementType::Adjustment | MovementType::Initial => {
                if quantity < 0 {
                    return Err(DomainError::ValidationError(
                        "Inbound, adjustment, and initial movements must have positive quantity"
                            .to_string(),
                    ));
                }
            }
            MovementType::Outbound | MovementType::Transfer => {
                if quantity > 0 {
                    return Err(DomainError::ValidationError(
                        "Outbound and transfer movements must have negative quantity".to_string(),
                    ));
                }
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            item_id,
            location_id,
            movement_type,
            quantity,
            reference_type,
            reference_id,
            reason,
            created_at: Utc::now(),
            created_by,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLevel {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub quantity_on_hand: i32,
    pub last_movement_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl StockLevel {
    pub fn new(item_id: Uuid, location_id: Uuid) -> Self {
        Self {
            item_id,
            location_id,
            quantity_on_hand: 0,
            last_movement_id: None,
            updated_at: Utc::now(),
        }
    }

    pub fn apply_movement(&mut self, movement: &StockMovement) -> Result<(), DomainError> {
        // Validate that the movement applies to this stock level
        if movement.item_id != self.item_id || movement.location_id != self.location_id {
            return Err(DomainError::ValidationError(
                "Movement does not apply to this stock level".to_string(),
            ));
        }

        self.quantity_on_hand += movement.quantity;
        self.last_movement_id = Some(movement.id);
        self.updated_at = Utc::now();

        // Ensure stock level doesn't go negative (except for adjustments)
        if self.quantity_on_hand < 0 && movement.movement_type != MovementType::Adjustment {
            return Err(DomainError::ValidationError(
                "Stock level cannot go negative".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStockMovementRequest {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub movement_type: String,
    pub quantity: i32,
    pub reference_type: String,
    pub reference_id: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockMovementResponse {
    pub id: Uuid,
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub movement_type: String,
    pub quantity: i32,
    pub reference_type: String,
    pub reference_id: Option<Uuid>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub item: Option<Item>,
    pub location: Option<Location>,
    pub created_by_user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockLevelResponse {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub quantity_on_hand: i32,
    pub last_movement_id: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub item: Option<Item>,
    pub location: Option<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StockAdjustmentRequest {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub quantity: i32,
    pub reason: String,
}
