use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SalesOrderStatus {
    Draft,
    Confirmed,
    Picking,
    Shipped,
    Invoiced,
    Cancelled,
    Returned,
}

impl SalesOrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SalesOrderStatus::Draft => "DRAFT",
            SalesOrderStatus::Confirmed => "CONFIRMED",
            SalesOrderStatus::Picking => "PICKING",
            SalesOrderStatus::Shipped => "SHIPPED",
            SalesOrderStatus::Invoiced => "INVOICED",
            SalesOrderStatus::Cancelled => "CANCELLED",
            SalesOrderStatus::Returned => "RETURNED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "DRAFT" => Ok(SalesOrderStatus::Draft),
            "CONFIRMED" => Ok(SalesOrderStatus::Confirmed),
            "PICKING" => Ok(SalesOrderStatus::Picking),
            "SHIPPED" => Ok(SalesOrderStatus::Shipped),
            "INVOICED" => Ok(SalesOrderStatus::Invoiced),
            "CANCELLED" => Ok(SalesOrderStatus::Cancelled),
            "RETURNED" => Ok(SalesOrderStatus::Returned),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid sales order status: {}",
                s
            ))),
        }
    }

    pub fn can_transition_to(&self, new_status: &SalesOrderStatus) -> bool {
        match self {
            SalesOrderStatus::Draft => matches!(
                new_status,
                SalesOrderStatus::Confirmed | SalesOrderStatus::Cancelled
            ),
            SalesOrderStatus::Confirmed => matches!(
                new_status,
                SalesOrderStatus::Picking | SalesOrderStatus::Cancelled
            ),
            SalesOrderStatus::Picking => matches!(
                new_status,
                SalesOrderStatus::Shipped | SalesOrderStatus::Cancelled
            ),
            SalesOrderStatus::Shipped => matches!(
                new_status,
                SalesOrderStatus::Invoiced | SalesOrderStatus::Returned
            ),
            SalesOrderStatus::Invoiced => matches!(new_status, SalesOrderStatus::Returned),
            SalesOrderStatus::Cancelled => false,
            SalesOrderStatus::Returned => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderLine {
    pub id: Uuid,
    pub so_id: Uuid,
    pub item_id: Uuid,
    pub qty: i32,
    pub unit_price: f64,
    pub tax: f64,
    pub reserved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SalesOrderLine {
    pub fn new(item_id: Uuid, qty: i32, unit_price: f64) -> Result<Self, DomainError> {
        if qty <= 0 {
            return Err(DomainError::ValidationError(
                "Quantity must be positive".to_string(),
            ));
        }
        if unit_price < 0.0 {
            return Err(DomainError::ValidationError(
                "Unit price cannot be negative".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            so_id: Uuid::new_v4(), // Will be set when added to order
            item_id,
            qty,
            unit_price,
            tax: 0.0,
            reserved: false,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn line_total(&self) -> f64 {
        (self.qty as f64 * self.unit_price) + self.tax
    }

    pub fn reserve(&mut self) -> Result<(), DomainError> {
        if self.reserved {
            return Err(DomainError::ValidationError(
                "Line is already reserved".to_string(),
            ));
        }
        self.reserved = true;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn unreserve(&mut self) -> Result<(), DomainError> {
        if !self.reserved {
            return Err(DomainError::ValidationError(
                "Line is not reserved".to_string(),
            ));
        }
        self.reserved = false;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    pub id: Uuid,
    pub so_number: String,
    pub customer_id: Option<Uuid>,
    pub status: SalesOrderStatus,
    pub total_amount: f64,
    pub fulfillment_location_id: Option<Uuid>,
    pub lines: Vec<SalesOrderLine>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SalesOrder {
    pub fn new(
        so_number: String,
        customer_id: Option<Uuid>,
        fulfillment_location_id: Option<Uuid>,
        created_by: Uuid,
    ) -> Result<Self, DomainError> {
        if so_number.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "SO number cannot be empty".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            so_number,
            customer_id,
            status: SalesOrderStatus::Draft,
            total_amount: 0.0,
            fulfillment_location_id,
            lines: Vec::new(),
            created_by,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn add_line(&mut self, mut line: SalesOrderLine) -> Result<(), DomainError> {
        if self.status != SalesOrderStatus::Draft {
            return Err(DomainError::ValidationError(
                "Cannot add lines to non-draft sales order".to_string(),
            ));
        }

        line.so_id = self.id;
        self.lines.push(line);
        self.recalculate_total();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn confirm(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&SalesOrderStatus::Confirmed) {
            return Err(DomainError::ValidationError(format!(
                "Cannot confirm sales order with status: {:?}",
                self.status
            )));
        }

        if self.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Cannot confirm sales order without lines".to_string(),
            ));
        }

        self.status = SalesOrderStatus::Confirmed;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn start_picking(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&SalesOrderStatus::Picking) {
            return Err(DomainError::ValidationError(format!(
                "Cannot start picking for sales order with status: {:?}",
                self.status
            )));
        }

        self.status = SalesOrderStatus::Picking;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn ship(
        &mut self,
        shipped_lines: Vec<ShipLineRequest>,
    ) -> Result<Vec<StockMovement>, DomainError> {
        // If order is confirmed, automatically start picking
        if self.status == SalesOrderStatus::Confirmed {
            self.start_picking()?;
        }

        if !self.status.can_transition_to(&SalesOrderStatus::Shipped) {
            return Err(DomainError::ValidationError(format!(
                "Cannot ship sales order with status: {:?}",
                self.status
            )));
        }

        if shipped_lines.is_empty() {
            return Err(DomainError::ValidationError(
                "No lines specified for shipping".to_string(),
            ));
        }

        let mut stock_movements = Vec::new();
        let fulfillment_location_id = self.fulfillment_location_id.ok_or_else(|| {
            DomainError::ValidationError("Fulfillment location required for shipping".to_string())
        })?;

        for ship_request in shipped_lines {
            let line = self
                .lines
                .iter_mut()
                .find(|l| l.id == ship_request.so_line_id)
                .ok_or_else(|| {
                    DomainError::ValidationError(format!(
                        "Line {} not found",
                        ship_request.so_line_id
                    ))
                })?;

            if ship_request.qty_shipped > line.qty {
                return Err(DomainError::ValidationError(format!(
                    "Cannot ship {} units of line {}, only {} ordered",
                    ship_request.qty_shipped, line.id, line.qty
                )));
            }

            if ship_request.qty_shipped <= 0 {
                return Err(DomainError::ValidationError(
                    "Shipped quantity must be positive".to_string(),
                ));
            }

            // Create stock movement for the shipment (outbound)
            let movement = StockMovement::new(
                line.item_id,
                fulfillment_location_id,
                MovementType::Outbound,
                -(ship_request.qty_shipped as i32), // Negative for outbound
                ReferenceType::SalesOrder,
                Some(self.id),
                Some(format!(
                    "Shipped {} units of item {}",
                    ship_request.qty_shipped, line.item_id
                )),
                Some(self.created_by),
            );

            stock_movements.push(movement?);

            // If line was reserved, unreserve it (assuming full shipment)
            if line.reserved {
                line.unreserve()?;
            }
        }

        self.status = SalesOrderStatus::Shipped;
        self.updated_at = Utc::now();
        Ok(stock_movements)
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&SalesOrderStatus::Cancelled) {
            return Err(DomainError::ValidationError(format!(
                "Cannot cancel sales order with status: {:?}",
                self.status
            )));
        }

        self.status = SalesOrderStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn reserve_inventory(&mut self) -> Result<Vec<StockMovement>, DomainError> {
        if self.status != SalesOrderStatus::Confirmed {
            return Err(DomainError::ValidationError(
                "Can only reserve inventory for confirmed sales orders".to_string(),
            ));
        }

        let fulfillment_location_id = self.fulfillment_location_id.ok_or_else(|| {
            DomainError::ValidationError(
                "Fulfillment location required for reservation".to_string(),
            )
        })?;

        let mut stock_movements = Vec::new();

        for line in &mut self.lines {
            if !line.reserved {
                // Create a reservation movement (this would typically be a soft reservation)
                // For now, we'll use an adjustment type to represent the reservation
                let movement = StockMovement::new(
                    line.item_id,
                    fulfillment_location_id,
                    MovementType::Adjustment,
                    0, // No actual quantity change for reservation
                    ReferenceType::SalesOrder,
                    Some(self.id),
                    Some(format!(
                        "Reserved {} units for sales order {}",
                        line.qty, self.so_number
                    )),
                    Some(self.created_by),
                );

                line.reserve()?;
                stock_movements.push(movement?);
            }
        }

        Ok(stock_movements)
    }

    fn recalculate_total(&mut self) {
        self.total_amount = self.lines.iter().map(|line| line.line_total()).sum();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipLineRequest {
    pub so_line_id: Uuid,
    pub qty_shipped: i32,
}

// Re-export for convenience
pub use crate::domain::entities::inventory::{MovementType, ReferenceType, StockMovement};
