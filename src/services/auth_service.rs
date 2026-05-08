use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::config::Config;
use crate::dto::auth_dto::{LoginResponse, RefreshResponse};
use crate::errors::AppError;
use crate::models::user::User;
use crate::repositories::{RefreshTokenRepository, UserRepository};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    refresh_repo: Arc<dyn RefreshTokenRepository>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        refresh_repo: Arc<dyn RefreshTokenRepository>,
    ) -> Self {
        Self {
            user_repo,
            refresh_repo,
        }
    }

    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_access_token(user: &User, config: &Config) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::hours(config.jwt_expiration_hours);

        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
    }

    pub fn verify_access_token(token: &str, config: &Config) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub async fn login(
        &self,
        config: &Config,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse, AppError> {
        let user = self
            .user_repo
            .find_by_email(email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if !Self::verify_password(password, &user.password_hash)? {
            return Err(AppError::Unauthorized);
        }

        let access_token = Self::generate_access_token(&user, config)?;
        let refresh_token = Uuid::new_v4().to_string();
        let refresh_token_hash = Self::hash_token(&refresh_token);

        let expires_at = Utc::now() + Duration::days(config.refresh_token_expiration_days);
        self.refresh_repo
            .create(user.id, &refresh_token_hash, expires_at)
            .await?;

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: config.jwt_expiration_hours * 3600,
        })
    }

    pub async fn refresh(
        &self,
        config: &Config,
        refresh_token: &str,
    ) -> Result<RefreshResponse, AppError> {
        let token_hash = Self::hash_token(refresh_token);

        let stored_token = self
            .refresh_repo
            .find_by_hash(&token_hash)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if stored_token.expires_at < Utc::now() {
            self.refresh_repo.revoke(stored_token.id).await?;
            return Err(AppError::Unauthorized);
        }

        let user = self
            .user_repo
            .find_by_id(stored_token.user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let access_token = Self::generate_access_token(&user, config)?;

        Ok(RefreshResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: config.jwt_expiration_hours * 3600,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        let token_hash = Self::hash_token(refresh_token);

        if let Some(stored_token) = self.refresh_repo.find_by_hash(&token_hash).await? {
            self.refresh_repo.revoke(stored_token.id).await?;
        }

        Ok(())
    }

    pub async fn logout_all(&self, user_id: Uuid) -> Result<u64, AppError> {
        let count = self.refresh_repo.revoke_all_for_user(user_id).await?;
        Ok(count)
    }
}
