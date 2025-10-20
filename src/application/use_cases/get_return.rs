use crate::domain::entities::returns::{Return, ReturnLine};
use crate::domain::services::return_repository::ReturnRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GetReturnResponse {
    pub return_entity: Return,
    pub lines: Vec<ReturnLine>,
}

pub struct GetReturnUseCase<R: ReturnRepository> {
    return_repository: Arc<R>,
}

impl<R: ReturnRepository> GetReturnUseCase<R> {
    pub fn new(return_repository: Arc<R>) -> Self {
        Self { return_repository }
    }

    pub async fn execute(&self, return_id: Uuid) -> Result<GetReturnResponse, DomainError> {
        let (return_entity, lines) = self
            .return_repository
            .find_by_id(return_id)
            .await?
            .ok_or_else(|| DomainError::NotFound(format!("Return {} not found", return_id)))?;

        Ok(GetReturnResponse { return_entity, lines })
    }

    pub async fn execute_by_number(
        &self,
        return_number: &str,
    ) -> Result<GetReturnResponse, DomainError> {
        let (return_entity, lines) = self
            .return_repository
            .find_by_return_number(return_number)
            .await?
            .ok_or_else(|| {
                DomainError::NotFound(format!("Return {} not found", return_number))
            })?;

        Ok(GetReturnResponse { return_entity, lines })
    }
}