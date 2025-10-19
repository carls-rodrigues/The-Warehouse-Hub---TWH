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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::user::User;
    use crate::domain::value_objects::{email::Email, password_hash::PasswordHash};
    use async_trait::async_trait;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock UserRepository for testing
    struct MockUserRepository {
        users: std::collections::HashMap<String, User>,
        should_fail: bool,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: std::collections::HashMap::new(),
                should_fail: false,
            }
        }

        fn with_user(mut self, user: User) -> Self {
            self.users.insert(user.email.as_str().to_string(), user);
            self
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Database error".to_string()));
            }
            Ok(None)
        }

        async fn find_by_email(&self, email: &Email) -> Result<Option<User>, DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError("Database error".to_string()));
            }
            Ok(self.users.get(email.as_str()).cloned())
        }

        async fn save(&self, _user: &User) -> Result<(), DomainError> {
            Ok(())
        }

        async fn update(&self, _user: &User) -> Result<(), DomainError> {
            Ok(())
        }

        async fn delete(&self, _id: Uuid) -> Result<(), DomainError> {
            Ok(())
        }

        async fn email_exists(
            &self,
            _email: &Email,
            _exclude_user_id: Option<Uuid>,
        ) -> Result<bool, DomainError> {
            Ok(false)
        }
    }

    fn create_test_user() -> User {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let password_hash = PasswordHash::from_hash("$2b$12$kQMaeUg9psmEZ/L6aS/d6.YrMZv5VCym3ebSX7D1.3lvT3iq6/wfC".to_string());
        User::new(
            email,
            password_hash,
            "Test".to_string(),
            "User".to_string(),
        ).unwrap()
    }

    #[tokio::test]
    async fn test_login_success() {
        let user = create_test_user();
        let mock_repo = MockUserRepository::new().with_user(user.clone());
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.user_id, user.id.to_string());
        assert!(!response.token.is_empty());
        assert!(response.expires_at > chrono::Utc::now().timestamp());
    }

    #[tokio::test]
    async fn test_login_invalid_email() {
        let mock_repo = MockUserRepository::new();
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "invalid-email".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ValidationError(_)));
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let mock_repo = MockUserRepository::new();
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "notfound@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DomainError::ValidationError(msg) if msg == "Invalid credentials"));
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let user = create_test_user();
        let mock_repo = MockUserRepository::new().with_user(user);
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DomainError::ValidationError(msg) if msg == "Invalid credentials"));
    }

    #[tokio::test]
    async fn test_login_deactivated_user() {
        let mut user = create_test_user();
        user.deactivate();
        let mock_repo = MockUserRepository::new().with_user(user);
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, DomainError::ValidationError(msg) if msg == "Account is deactivated"));
    }

    #[tokio::test]
    async fn test_login_database_error() {
        let mock_repo = MockUserRepository::new().with_failure();
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_jwt_token_structure() {
        let user = create_test_user();
        let mock_repo = MockUserRepository::new().with_user(user.clone());
        let use_case = LoginUseCase::new(
            Arc::new(mock_repo),
            "test-secret".to_string(),
            24,
        );

        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
        };

        let result = use_case.execute(request).await.unwrap();

        // Decode the JWT token to verify its structure
        use jsonwebtoken::{decode, DecodingKey, Validation};
        let token_data = decode::<Claims>(
            &result.token,
            &DecodingKey::from_secret("test-secret".as_ref()),
            &Validation::default(),
        ).unwrap();

        assert_eq!(token_data.claims.sub, user.id.to_string());
        assert_eq!(token_data.claims.email, "test@example.com");
        assert!(token_data.claims.exp > token_data.claims.iat);
    }
}
