use crate::shared::error::DomainError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ItemDimensions {
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItemRequest {
    pub sku: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: Option<String>,
    pub barcode: Option<String>,
    pub cost_price: Option<f64>,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<ItemDimensions>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub barcode: Option<String>,
    pub cost_price: f64,
    pub sale_price: Option<f64>,
    pub reorder_point: Option<i32>,
    pub reorder_qty: Option<i32>,
    pub weight: Option<f64>,
    pub dimensions: Option<ItemDimensions>,
    pub metadata: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Item {
    pub fn new(
        tenant_id: Uuid,
        sku: String,
        name: String,
        unit: String,
        cost_price: f64,
    ) -> Result<Self, DomainError> {
        if sku.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "SKU cannot be empty".to_string(),
            ));
        }

        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        if unit.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Unit cannot be empty".to_string(),
            ));
        }

        if cost_price < 0.0 {
            return Err(DomainError::ValidationError(
                "Cost price cannot be negative".to_string(),
            ));
        }

        let now = chrono::Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            tenant_id,
            sku,
            name,
            description: None,
            category: None,
            unit,
            barcode: None,
            cost_price,
            sale_price: None,
            reorder_point: None,
            reorder_qty: None,
            weight: None,
            dimensions: None,
            metadata: None,
            active: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update(&mut self, request: UpdateItemRequest) -> Result<(), DomainError> {
        if let Some(sku) = request.sku {
            if sku.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "SKU cannot be empty".to_string(),
                ));
            }
            self.sku = sku;
        }

        if let Some(name) = request.name {
            if name.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Name cannot be empty".to_string(),
                ));
            }
            self.name = name;
        }

        if let Some(description) = request.description {
            self.description = Some(description);
        }

        if let Some(category) = request.category {
            self.category = Some(category);
        }

        if let Some(unit) = request.unit {
            if unit.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Unit cannot be empty".to_string(),
                ));
            }
            self.unit = unit;
        }

        if let Some(barcode) = request.barcode {
            self.barcode = Some(barcode);
        }

        if let Some(cost_price) = request.cost_price {
            if cost_price < 0.0 {
                return Err(DomainError::ValidationError(
                    "Cost price cannot be negative".to_string(),
                ));
            }
            self.cost_price = cost_price;
        }

        if let Some(sale_price) = request.sale_price {
            if sale_price < 0.0 {
                return Err(DomainError::ValidationError(
                    "Sale price cannot be negative".to_string(),
                ));
            }
            self.sale_price = Some(sale_price);
        }

        if let Some(reorder_point) = request.reorder_point {
            if reorder_point < 0 {
                return Err(DomainError::ValidationError(
                    "Reorder point cannot be negative".to_string(),
                ));
            }
            self.reorder_point = Some(reorder_point);
        }

        if let Some(reorder_qty) = request.reorder_qty {
            if reorder_qty < 0 {
                return Err(DomainError::ValidationError(
                    "Reorder quantity cannot be negative".to_string(),
                ));
            }
            self.reorder_qty = Some(reorder_qty);
        }

        if let Some(weight) = request.weight {
            if weight < 0.0 {
                return Err(DomainError::ValidationError(
                    "Weight cannot be negative".to_string(),
                ));
            }
            self.weight = Some(weight);
        }

        if let Some(dimensions) = request.dimensions {
            self.dimensions = Some(dimensions);
        }

        if let Some(metadata) = request.metadata {
            self.metadata = Some(metadata);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = chrono::Utc::now();
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = chrono::Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn full_name(&self) -> String {
        format!("{} ({})", self.name, self.sku)
    }
}
