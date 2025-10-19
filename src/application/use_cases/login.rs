use crate::domain::services::user_repository::UserRepository;
use crate::domain::value_objects::email::Email;
use crate::shared::error::DomainError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub email: String,
    pub expires_at: i64,
}

pub struct LoginUseCase<R: UserRepository> {
    user_repository: Arc<R>,
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

impl<R: UserRepository> LoginUseCase<R> {
    pub fn new(user_repository: Arc<R>, jwt_secret: String, jwt_expiry_hours: i64) -> Self {
        Self {
            user_repository,
            jwt_secret,
            jwt_expiry_hours,
        }
    }

    pub async fn execute(&self, request: LoginRequest) -> Result<LoginResponse, DomainError> {
        // Validate email format
        let email = Email::new(request.email)?;

        // Find user by email
        let user = self
            .user_repository
            .find_by_email(&email)
            .await?
            .ok_or_else(|| DomainError::ValidationError("Invalid credentials".to_string()))?;

        // Check if user is active
        if !user.is_active() {
            return Err(DomainError::ValidationError(
                "Account is deactivated".to_string(),
            ));
        }

        // Verify password
        let password_valid = user.verify_password(&request.password)?;
        if !password_valid {
            return Err(DomainError::ValidationError(
                "Invalid credentials".to_string(),
            ));
        }

        // Generate JWT token
        let token = self.generate_jwt(&user)?;

        let expires_at =
            (chrono::Utc::now() + chrono::Duration::hours(self.jwt_expiry_hours)).timestamp();

        Ok(LoginResponse {
            token,
            user_id: user.id.to_string(),
            email: user.email.as_str().to_string(),
            expires_at,
        })
    }

    fn generate_jwt(
        &self,
        user: &crate::domain::entities::user::User,
    ) -> Result<String, DomainError> {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(self.jwt_expiry_hours))
            .ok_or_else(|| {
                DomainError::ValidationError("Failed to calculate token expiration".to_string())
            })?;

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.as_str().to_string(),
            exp: expiration.timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(|_| DomainError::ValidationError("Failed to generate token".to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,   // User ID
    email: String, // User email
    exp: usize,    // Expiration time
    iat: usize,    // Issued at time
}
