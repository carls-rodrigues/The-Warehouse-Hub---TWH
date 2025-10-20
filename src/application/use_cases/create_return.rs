use crate::domain::entities::returns::{CreateReturnRequest, Return, ReturnLine};
use crate::domain::services::return_repository::ReturnRepository;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateReturnResponse {
    pub return_entity: Return,
}

pub struct CreateReturnUseCase<R: ReturnRepository> {
    return_repository: Arc<R>,
}

impl<R: ReturnRepository> CreateReturnUseCase<R> {
    pub fn new(return_repository: Arc<R>) -> Self {
        Self { return_repository }
    }

    pub async fn execute(
        &self,
        request: CreateReturnRequest,
        created_by: Uuid,
    ) -> Result<CreateReturnResponse, DomainError> {
        // Validate request
        if request.lines.is_empty() {
            return Err(DomainError::ValidationError(
                "Return must have at least one line".to_string(),
            ));
        }

        // Generate return number (in a real app, this might come from a sequence)
        let return_number = format!("RT-{}", Uuid::new_v4().simple());

        // Create return
        let mut return_entity = Return::new(
            return_number,
            request.customer_id,
            request.location_id,
            created_by,
        )?;

        // Set notes if provided
        return_entity.notes = request.notes;

        // Add lines
        for line_req in request.lines {
            let line = ReturnLine::new(
                return_entity.id,
                line_req.item_id,
                line_req.quantity,
                line_req.unit_price,
                line_req.reason,
            )?;
            return_entity.add_line(line)?;
        }

        // Create in repository
        self.return_repository.create(&return_entity).await?;

        Ok(CreateReturnResponse { return_entity })
    }
}
