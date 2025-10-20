use crate::domain::entities::inventory::StockMovement;
use crate::domain::entities::returns::{ProcessReturnRequest, Return, ReturnLine};
use crate::domain::services::return_repository::ReturnRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ProcessReturnResponse {
    pub return_entity: Return,
    pub lines: Vec<ReturnLine>,
    pub stock_movements: Vec<StockMovement>,
}

pub struct ProcessReturnUseCase<R: ReturnRepository> {
    return_repository: Arc<R>,
}

impl<R: ReturnRepository> ProcessReturnUseCase<R> {
    pub fn new(return_repository: Arc<R>) -> Self {
        Self { return_repository }
    }

    pub async fn execute(
        &self,
        return_id: Uuid,
        request: ProcessReturnRequest,
        created_by: Uuid,
    ) -> Result<ProcessReturnResponse, DomainError> {
        // Process the return through the repository
        let (return_entity, lines, stock_movements) = self
            .return_repository
            .process_return(return_id, request, created_by)
            .await?;

        Ok(ProcessReturnResponse {
            return_entity,
            lines,
            stock_movements,
        })
    }
}
