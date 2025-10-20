use crate::shared::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PurchaseOrderStatus {
    Draft,
    Open,
    Receiving,
    PartialReceived,
    Received,
    Cancelled,
}

impl std::fmt::Display for PurchaseOrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PurchaseOrderStatus::Draft => write!(f, "DRAFT"),
            PurchaseOrderStatus::Open => write!(f, "OPEN"),
            PurchaseOrderStatus::Receiving => write!(f, "RECEIVING"),
            PurchaseOrderStatus::PartialReceived => write!(f, "PARTIAL_RECEIVED"),
            PurchaseOrderStatus::Received => write!(f, "RECEIVED"),
            PurchaseOrderStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderLine {
    pub id: Uuid,
    pub po_id: Uuid,
    pub item_id: Uuid,
    pub qty_ordered: i32,
    pub qty_received: i32,
    pub unit_cost: f64,
    pub line_total: f64,
}

impl PurchaseOrderLine {
    pub fn new(item_id: Uuid, qty_ordered: i32, unit_cost: f64) -> Result<Self, DomainError> {
        if qty_ordered <= 0 {
            return Err(DomainError::ValidationError(
                "Quantity ordered must be positive".to_string(),
            ));
        }

        if unit_cost < 0.0 {
            return Err(DomainError::ValidationError(
                "Unit cost cannot be negative".to_string(),
            ));
        }

        let line_total = qty_ordered as f64 * unit_cost;

        Ok(Self {
            id: Uuid::new_v4(),
            po_id: Uuid::nil(), // Will be set when added to PO
            item_id,
            qty_ordered,
            qty_received: 0,
            unit_cost,
            line_total,
        })
    }

    pub fn receive(&mut self, qty: i32) -> Result<(), DomainError> {
        if qty <= 0 {
            return Err(DomainError::ValidationError(
                "Receive quantity must be positive".to_string(),
            ));
        }

        if self.qty_received + qty > self.qty_ordered {
            return Err(DomainError::ValidationError(
                "Cannot receive more than ordered".to_string(),
            ));
        }

        self.qty_received += qty;
        Ok(())
    }

    pub fn is_fully_received(&self) -> bool {
        self.qty_received >= self.qty_ordered
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePurchaseOrderRequest {
    pub supplier_id: Uuid,
    pub expected_date: Option<DateTime<Utc>>,
    pub lines: Vec<CreatePurchaseOrderLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePurchaseOrderLine {
    pub item_id: Uuid,
    pub qty_ordered: i32,
    pub unit_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    pub id: Uuid,
    pub po_number: String,
    pub supplier_id: Uuid,
    pub status: PurchaseOrderStatus,
    pub expected_date: Option<DateTime<Utc>>,
    pub total_amount: f64,
    pub lines: Vec<PurchaseOrderLine>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PurchaseOrder {
    pub fn new(
        supplier_id: Uuid,
        lines: Vec<CreatePurchaseOrderLine>,
        expected_date: Option<DateTime<Utc>>,
        created_by: Uuid,
    ) -> Result<Self, DomainError> {
        if lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Purchase order must have at least one line".to_string(),
            ));
        }

        let mut po_lines = Vec::new();
        let mut total_amount = 0.0;

        for line_req in lines {
            let mut line = PurchaseOrderLine::new(line_req.item_id, line_req.qty_ordered, line_req.unit_cost)?;
            line.po_id = Uuid::nil(); // Will be set after PO creation
            total_amount += line.line_total;
            po_lines.push(line);
        }

        let now = Utc::now();
        let po_number = format!("PO-{}", now.timestamp());

        let mut po = Self {
            id: Uuid::new_v4(),
            po_number,
            supplier_id,
            status: PurchaseOrderStatus::Draft,
            expected_date,
            total_amount,
            lines: po_lines,
            created_by,
            created_at: now,
            updated_at: now,
        };

        // Set po_id on lines
        for line in &mut po.lines {
            line.po_id = po.id;
        }

        Ok(po)
    }

    pub fn open(&mut self) -> Result<(), DomainError> {
        if self.status != PurchaseOrderStatus::Draft {
            return Err(DomainError::ValidationError(
                "Only draft purchase orders can be opened".to_string(),
            ));
        }
        self.status = PurchaseOrderStatus::Open;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status == PurchaseOrderStatus::Received || self.status == PurchaseOrderStatus::Cancelled {
            return Err(DomainError::ValidationError(
                "Cannot cancel a received or already cancelled purchase order".to_string(),
            ));
        }
        self.status = PurchaseOrderStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn receive_lines(&mut self, received_lines: Vec<ReceiveLine>) -> Result<(), DomainError> {
        if self.status == PurchaseOrderStatus::Cancelled || self.status == PurchaseOrderStatus::Received {
            return Err(DomainError::ValidationError(
                "Cannot receive lines on cancelled or fully received purchase order".to_string(),
            ));
        }

        self.status = PurchaseOrderStatus::Receiving;

        for receive_req in received_lines {
            let line = self.lines.iter_mut()
                .find(|l| l.id == receive_req.po_line_id)
                .ok_or_else(|| DomainError::ValidationError(
                    format!("Purchase order line {} not found", receive_req.po_line_id),
                ))?;

            line.receive(receive_req.qty_received)?;
        }

        // Update status based on receipt
        let all_received = self.lines.iter().all(|l| l.is_fully_received());
        let any_received = self.lines.iter().any(|l| l.qty_received > 0);

        if all_received {
            self.status = PurchaseOrderStatus::Received;
        } else if any_received {
            self.status = PurchaseOrderStatus::PartialReceived;
        } else {
            self.status = PurchaseOrderStatus::Open;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_fully_received(&self) -> bool {
        self.lines.iter().all(|l| l.is_fully_received())
    }

    pub fn total_received_qty(&self) -> i32 {
        self.lines.iter().map(|l| l.qty_received).sum()
    }

    pub fn total_ordered_qty(&self) -> i32 {
        self.lines.iter().map(|l| l.qty_ordered).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveLine {
    pub po_line_id: Uuid,
    pub qty_received: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivePurchaseOrderRequest {
    pub received_lines: Vec<ReceiveLine>,
    pub receive_date: Option<DateTime<Utc>>,
    pub destination_location_id: Uuid,
}