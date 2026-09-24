use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use chrono::Utc;

use crate::auth::types::Claims;
use axum::http::{HeaderMap, StatusCode};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET var should be set")
}

pub fn hash_password(password: &str) -> Result<String, StatusCode> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, StatusCode> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Creates a signed JWT valid for 1 hour.
/// Keeps `sub` as the email for display and `user_id` as the UUID used for auth scoping.
pub fn create_jwt(email: &str, user_id: &str) -> Result<String, StatusCode> {
    let claims = Claims {
        sub: email.to_string(),
        exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        user_id: user_id.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_ref()),
    )
    .map_err(|e| {
        eprintln!("Error generating token {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

/// Validates JWT signature and requires a non-expired `exp` claim.
/// Missing or expired `exp` maps to 401 so callers never accept stale tokens.
pub fn validate_jwt(token: &str) -> Result<Claims, StatusCode> {
    let mut validation = Validation::default();
    validation.set_required_spec_claims(&["exp"]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        eprintln!("Error validating token {}", e);
        StatusCode::UNAUTHORIZED
    })
}

/// Extracts the raw token from an `Authorization: Bearer <token>` header.
/// Centralizes prefix stripping so routes reject malformed headers with 401 instead of panicking.
pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .ok_or(StatusCode::UNAUTHORIZED)
}

/// Validates the token and returns the `user_id` claim as UUID.
/// Uses `user_id` (not `sub`, which holds the email) so valid login tokens parse correctly.
pub fn extract_user_id_from_token(token: &str) -> Result<uuid::Uuid, StatusCode> {
    let claims = validate_jwt(token)?;
    claims
        .user_id
        .parse::<uuid::Uuid>()
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::types::RegisterInfo;
    use validator::Validate;

    fn setup() {
        unsafe { std::env::set_var("JWT_SECRET", "testsecret") };
    }

    #[test]
    fn hash_password_and_verify_roundtrip() {
        setup();
        let password = "test_password_123";
        let hash = hash_password(password).expect("hashing should succeed");
        let valid = verify_password(password, &hash).expect("verification should succeed");
        assert!(valid);

        let invalid = verify_password("wrong_password", &hash).expect("verification should succeed");
        assert!(!invalid);
    }

    #[test]
    fn create_jwt_and_validate_roundtrip() {
        setup();
        let email = "test@example.com";
        let user_id = "123e4567-e89b-12d3-a456-426614174000"; // UUID as string
        let token = create_jwt(email, user_id).expect("JWT creation should succeed");
        let claims = validate_jwt(&token).expect("JWT validation should succeed");
        assert_eq!(claims.sub, email);
        assert_eq!(claims.user_id, user_id);
    }

    #[test]
    fn validate_jwt_rejects_invalid_token() {
        setup();
        let result = validate_jwt("invalid.token.here");
        assert!(matches!(result, Err(StatusCode::UNAUTHORIZED)));
    }

    #[test]
    fn validate_jwt_rejects_expired_token() {
        setup();
        // Create a token with an expiration time in the past
        let claims = Claims {
            sub: "test@example.com".to_string(),
            exp: (Utc::now() - chrono::Duration::hours(1)).timestamp() as usize, // expired 1 hour ago
            user_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret().as_ref()),
        )
        .expect("token creation should succeed");
        // This token should be rejected because it's expired
        let result = validate_jwt(&token);
        assert!(matches!(result, Err(StatusCode::UNAUTHORIZED)));
    }

    #[test]
    fn validate_register_input_valid() {
        let info = RegisterInfo {
            username: "validuser".to_string(),
            email: "valid@example.com".to_string(),
            password: "password123".to_string(),
        };
        let result = info.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn validate_register_input_invalid_email() {
        let info = RegisterInfo {
            username: "validuser".to_string(),
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };
        let result = info.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_register_input_short_password() {
        let info = RegisterInfo {
            username: "validuser".to_string(),
            email: "valid@example.com".to_string(),
            password: "short".to_string(),
        };
        let result = info.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_register_input_empty_username() {
        let info = RegisterInfo {
            username: "".to_string(),
            email: "valid@example.com".to_string(),
            password: "password123".to_string(),
        };
        let result = info.validate();
        assert!(result.is_err());
    }
}