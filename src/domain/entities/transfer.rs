use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Draft,
    Open,
    InTransit,
    Received,
    Cancelled,
}

impl TransferStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferStatus::Draft => "DRAFT",
            TransferStatus::Open => "OPEN",
            TransferStatus::InTransit => "IN_TRANSIT",
            TransferStatus::Received => "RECEIVED",
            TransferStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, DomainError> {
        match s {
            "DRAFT" => Ok(TransferStatus::Draft),
            "OPEN" => Ok(TransferStatus::Open),
            "IN_TRANSIT" => Ok(TransferStatus::InTransit),
            "RECEIVED" => Ok(TransferStatus::Received),
            "CANCELLED" => Ok(TransferStatus::Cancelled),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid transfer status: {}",
                s
            ))),
        }
    }

    pub fn can_transition_to(&self, new_status: &TransferStatus) -> bool {
        match self {
            TransferStatus::Draft => {
                matches!(new_status, TransferStatus::Open | TransferStatus::Cancelled)
            }
            TransferStatus::Open => matches!(
                new_status,
                TransferStatus::InTransit | TransferStatus::Cancelled
            ),
            TransferStatus::InTransit => matches!(
                new_status,
                TransferStatus::Received | TransferStatus::Cancelled
            ),
            TransferStatus::Received => false,
            TransferStatus::Cancelled => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: Uuid,
    pub transfer_number: String,
    pub from_location_id: Uuid,
    pub to_location_id: Uuid,
    pub status: TransferStatus,
    pub total_quantity: i32,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lines: Vec<TransferLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferLine {
    pub id: Uuid,
    pub transfer_id: Uuid,
    pub item_id: Uuid,
    pub quantity: i32,
    pub quantity_received: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransferRequest {
    pub from_location_id: Uuid,
    pub to_location_id: Uuid,
    pub lines: Vec<CreateTransferLineRequest>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransferLineRequest {
    pub item_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveTransferRequest {
    pub lines: Vec<ReceiveTransferLineRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveTransferLineRequest {
    pub transfer_line_id: Uuid,
    pub quantity_received: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipLineRequest {
    pub so_line_id: Uuid,
    pub qty_shipped: i32,
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
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReferenceType {
    PurchaseOrder,
    SalesOrder,
    Transfer,
    Adjustment,
    Initial,
}

impl ReferenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReferenceType::PurchaseOrder => "purchase_order",
            ReferenceType::SalesOrder => "sales_order",
            ReferenceType::Transfer => "transfer",
            ReferenceType::Adjustment => "adjustment",
            ReferenceType::Initial => "initial",
        }
    }
}

impl Transfer {
    pub fn new(
        transfer_number: String,
        from_location_id: Uuid,
        to_location_id: Uuid,
        created_by: Uuid,
    ) -> Result<Self, DomainError> {
        if from_location_id == to_location_id {
            return Err(DomainError::ValidationError(
                "From and to locations cannot be the same".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transfer_number,
            from_location_id,
            to_location_id,
            status: TransferStatus::Draft,
            total_quantity: 0,
            notes: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            lines: Vec::new(),
        })
    }

    pub fn add_line(&mut self, line: TransferLine) -> Result<(), DomainError> {
        if line.quantity <= 0 {
            return Err(DomainError::ValidationError(
                "Transfer line quantity must be positive".to_string(),
            ));
        }

        self.lines.push(line);
        self.total_quantity = self.lines.iter().map(|l| l.quantity).sum();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn open(&mut self) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&TransferStatus::Open) {
            return Err(DomainError::ValidationError(format!(
                "Cannot open transfer with status: {:?}",
                self.status
            )));
        }

        self.status = TransferStatus::Open;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn ship(&mut self) -> Result<Vec<StockMovement>, DomainError> {
        if !self.status.can_transition_to(&TransferStatus::InTransit) {
            return Err(DomainError::ValidationError(format!(
                "Cannot ship transfer with status: {:?}",
                self.status
            )));
        }

        if self.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Transfer must have lines to ship".to_string(),
            ));
        }

        let mut stock_movements = Vec::new();

        // Create outbound movements from source location
        for line in &self.lines {
            let movement = StockMovement::new(
                line.item_id,
                self.from_location_id,
                MovementType::Outbound,
                -(line.quantity as i32), // Negative for outbound
                ReferenceType::Transfer,
                Some(self.id),
                format!(
                    "Transfer outbound: {} units of item {}",
                    line.quantity, line.item_id
                ),
                Utc::now(),
                self.created_by,
            );
            stock_movements.push(movement);
        }

        self.status = TransferStatus::InTransit;
        self.updated_at = Utc::now();
        Ok(stock_movements)
    }

    pub fn receive(
        &mut self,
        received_lines: Vec<ReceiveTransferLineRequest>,
    ) -> Result<Vec<StockMovement>, DomainError> {
        if self.status != TransferStatus::InTransit {
            return Err(DomainError::ValidationError(format!(
                "Cannot receive transfer with status: {:?}",
                self.status
            )));
        }

        if received_lines.is_empty() {
            return Err(DomainError::ValidationError(
                "No lines specified for receiving".to_string(),
            ));
        }

        let mut stock_movements = Vec::new();

        for receive_request in received_lines {
            let line = self
                .lines
                .iter_mut()
                .find(|l| l.id == receive_request.transfer_line_id)
                .ok_or_else(|| {
                    DomainError::ValidationError(format!(
                        "Line {} not found",
                        receive_request.transfer_line_id
                    ))
                })?;

            if receive_request.quantity_received > line.quantity {
                return Err(DomainError::ValidationError(format!(
                    "Cannot receive {} units of line {}, only {} ordered",
                    receive_request.quantity_received, line.id, line.quantity
                )));
            }

            if receive_request.quantity_received <= 0 {
                return Err(DomainError::ValidationError(
                    "Received quantity must be positive".to_string(),
                ));
            }

            line.quantity_received = receive_request.quantity_received;
            line.updated_at = Utc::now();

            // Create inbound movement to destination location
            let movement = StockMovement::new(
                line.item_id,
                self.to_location_id,
                MovementType::Inbound,
                receive_request.quantity_received,
                ReferenceType::Transfer,
                Some(self.id),
                format!(
                    "Transfer inbound: {} units of item {}",
                    receive_request.quantity_received, line.item_id
                ),
                Utc::now(),
                self.created_by,
            );
            stock_movements.push(movement);
        }

        // Check if all lines are fully received
        let total_received: i32 = self.lines.iter().map(|l| l.quantity_received).sum();
        let total_ordered: i32 = self.lines.iter().map(|l| l.quantity).sum();

        if total_received >= total_ordered {
            self.status = TransferStatus::Received;
        }

        self.updated_at = Utc::now();
        Ok(stock_movements)
    }
}

impl TransferLine {
    pub fn new(transfer_id: Uuid, item_id: Uuid, quantity: i32) -> Result<Self, DomainError> {
        if quantity <= 0 {
            return Err(DomainError::ValidationError(
                "Transfer line quantity must be positive".to_string(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            transfer_id,
            item_id,
            quantity,
            quantity_received: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl StockMovement {
    pub fn new(
        item_id: Uuid,
        location_id: Uuid,
        movement_type: MovementType,
        quantity: i32,
        reference_type: ReferenceType,
        reference_id: Option<Uuid>,
        reason: String,
        created_at: DateTime<Utc>,
        created_by: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            item_id,
            location_id,
            movement_type,
            quantity,
            reference_type,
            reference_id,
            reason,
            created_at,
            created_by,
        }
    }
}
