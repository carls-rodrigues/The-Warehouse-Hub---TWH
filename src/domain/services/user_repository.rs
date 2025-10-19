use crate::domain::entities::user::User;
use crate::domain::value_objects::email::Email;
use crate::shared::error::DomainError;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;

    /// Find a user by their email address
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError>;

    /// Save a new user
    async fn save(&self, user: &User) -> Result<(), DomainError>;

    /// Update an existing user
    async fn update(&self, user: &User) -> Result<(), DomainError>;

    /// Delete a user by ID
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;

    /// Check if an email is already taken by another user
    async fn email_exists(
        &self,
        email: &Email,
        exclude_user_id: Option<Uuid>,
    ) -> Result<bool, DomainError>;
}
