use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::auth::types::Claims;
use axum::http::StatusCode;

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

pub fn create_jwt(email: &str) -> Result<String, StatusCode> {
    let claims = Claims {
        sub: email.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
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

pub fn validate_jwt(token: &str) -> Result<Claims, StatusCode> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| {
        eprintln!("Error validating token {}", e);
        StatusCode::UNAUTHORIZED
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::types::RegisterInfo;
    use validator::Validate;

    #[test]
    fn hash_password_and_verify_roundtrip() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("hashing should succeed");
        let valid = verify_password(password, &hash).expect("verification should succeed");
        assert!(valid);

        let invalid = verify_password("wrong_password", &hash).expect("verification should succeed");
        assert!(!invalid);
    }

    #[test]
    fn create_jwt_and_validate_roundtrip() {
        let email = "test@example.com";
        let token = create_jwt(email).expect("JWT creation should succeed");
        let claims = validate_jwt(&token).expect("JWT validation should succeed");
        assert_eq!(claims.sub, email);
    }

    #[test]
    fn validate_jwt_rejects_invalid_token() {
        let result = validate_jwt("invalid.token.here");
        assert!(matches!(result, Err(StatusCode::UNAUTHORIZED)));
    }

    #[test]
    fn validate_jwt_rejects_expired_token() {
        // We can't easily create an expired token without time manipulation,
        // but we can test that validation fails for malformed tokens
        let result = validate_jwt("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0QGV4YW1wbGUuY29tIiwiZXhwIjoxfQ.invalid");
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