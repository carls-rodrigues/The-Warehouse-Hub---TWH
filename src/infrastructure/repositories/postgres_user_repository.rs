use crate::domain::entities::user::User;
use crate::domain::services::user_repository::UserRepository;
use crate::domain::value_objects::{email::Email, password_hash::PasswordHash};
use crate::shared::error::DomainError;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

impl PostgresUserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let email = Email::new(row.email).map_err(|_| {
                    DomainError::ValidationError("Invalid email in database".to_string())
                })?;
                let password_hash = PasswordHash::from_hash(row.password_hash);

                Ok(Some(User {
                    id: row.id,
                    email,
                    password_hash,
                    first_name: row.first_name.unwrap_or_default(),
                    last_name: row.last_name.unwrap_or_default(),
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
        let result = sqlx::query!(
            r#"
            SELECT id, email, password_hash, first_name, last_name, active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        match result {
            Some(row) => {
                let email = Email::new(row.email).map_err(|_| {
                    DomainError::ValidationError("Invalid email in database".to_string())
                })?;
                let password_hash = PasswordHash::from_hash(row.password_hash);

                Ok(Some(User {
                    id: row.id,
                    email,
                    password_hash,
                    first_name: row.first_name.unwrap_or_default(),
                    last_name: row.last_name.unwrap_or_default(),
                    active: row.active,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            user.id,
            user.email.as_str(),
            user.password_hash.as_str(),
            user.first_name,
            user.last_name,
            user.active,
            user.created_at,
            user.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            UPDATE users
            SET email = $2, password_hash = $3, first_name = $4, last_name = $5, active = $6, updated_at = $7
            WHERE id = $1
            "#,
            user.id,
            user.email.as_str(),
            user.password_hash.as_str(),
            user.first_name,
            user.last_name,
            user.active,
            user.updated_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query!(
            r#"
            DELETE FROM users WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn email_exists(
        &self,
        email: &Email,
        exclude_user_id: Option<Uuid>,
    ) -> Result<bool, DomainError> {
        let count: Option<i64> = if let Some(exclude_id) = exclude_user_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM users WHERE email = $1 AND id != $2",
                email.as_str(),
                exclude_id
            )
            .fetch_one(&*self.pool)
            .await
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM users WHERE email = $1",
                email.as_str()
            )
            .fetch_one(&*self.pool)
            .await
        }
        .map_err(|e| DomainError::ValidationError(format!("Database error: {}", e)))?;

        Ok(count.unwrap_or(0) > 0)
    }
}
