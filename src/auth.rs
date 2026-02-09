use crate::error::RpcError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: usize,  // Expiry timestamp
    pub iat: usize,  // Issued at
    pub role: String, // User role (e.g., "admin", "user")
}

/// Generate a JWT token
pub fn generate_token(user_id: &str, secret: &str, role: &str) -> Result<String, RpcError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let expiry = now + 3600 * 24; // 24 hours

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiry,
        iat: now,
        role: role.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| RpcError::Auth(format!("Failed to generate token: {}", e)))
}

/// Validate a JWT token
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, RpcError> {
    let validation = Validation::default();

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| RpcError::Auth(format!("Invalid token: {}", e)))
}

/// Middleware skeleton for JWT authentication
/// Usage: Add as layer to protected routes
pub async fn require_auth(
    // Extract Authorization header
    // Validate JWT
    // Pass through if valid, return 401 if not
) -> Result<(), RpcError> {
    // Implementation placeholder
    // In production, this would extract the Bearer token from headers
    // and validate it using validate_token()
    Ok(())
}
