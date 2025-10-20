use crate::shared::error::DomainError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationAddress {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub address: Option<LocationAddress>,
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    #[serde(rename = "warehouse")]
    Warehouse,
    #[serde(rename = "store")]
    Store,
    #[serde(rename = "drop-ship")]
    DropShip,
}

impl LocationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LocationType::Warehouse => "warehouse",
            LocationType::Store => "store",
            LocationType::DropShip => "drop-ship",
        }
    }

    pub fn from_str<S: AsRef<str>>(s: S) -> Result<Self, DomainError> {
        match s.as_ref() {
            "warehouse" => Ok(LocationType::Warehouse),
            "store" => Ok(LocationType::Store),
            "drop-ship" => Ok(LocationType::DropShip),
            _ => Err(DomainError::ValidationError(format!(
                "Invalid location type: {}. Must be one of: warehouse, store, drop-ship",
                s.as_ref()
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub address: Option<LocationAddress>,
    pub r#type: Option<LocationType>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Location {
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        let now = chrono::Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            name,
            code: None,
            address: None,
            r#type: None,
            active: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update(&mut self, request: UpdateLocationRequest) -> Result<(), DomainError> {
        if let Some(name) = request.name {
            if name.trim().is_empty() {
                return Err(DomainError::ValidationError(
                    "Name cannot be empty".to_string(),
                ));
            }
            self.name = name;
        }

        if let Some(code) = request.code {
            self.code = Some(code);
        }

        if let Some(address) = request.address {
            self.address = Some(address);
        }

        if let Some(type_str) = request.r#type {
            self.r#type = Some(LocationType::from_str(&type_str)?);
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
        if let Some(code) = &self.code {
            format!("{} ({})", self.name, code)
        } else {
            self.name.clone()
        }
    }
}
