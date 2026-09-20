use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::result::Result::Ok;

use validator::Validate;

use crate::auth::types::Claims;
use crate::repositories::user::UserRepository;

use super::types::{LoginInfo, LoginResponse, RegisterInfo, RegisterResponse};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET var should be set")
}

pub async fn login_handler(
    repo: &mut dyn UserRepository,
    Json(login_info): Json<LoginInfo>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = match repo.find_by_email(&login_info.email).await {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    if !Argon2::default()
        .verify_password(login_info.password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let claims = Claims {
        sub: login_info.email.clone(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };
    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_ref()),
    ) {
        Ok(tok) => tok,
        Err(e) => {
            eprintln!("Error generating token {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok(Json(LoginResponse { token }))
}

pub async fn register_handler(
    repo: &mut dyn UserRepository,
    Json(register_info): Json<RegisterInfo>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    register_info.validate().map_err(|_| StatusCode::BAD_REQUEST)?;
    if repo.email_exists(&register_info.email).await {
        return Err(StatusCode::CONFLICT);
    }
    if repo.username_exists(&register_info.username).await {
        return Err(StatusCode::CONFLICT);
    }
    let hash = Argon2::default()
        .hash_password(register_info.password.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let id = repo
        .create(&register_info.username, &register_info.email, &hash)
        .await
        .map_err(|e| {
            eprintln!("Error creating user {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(RegisterResponse { id: id.to_string() }))
}

pub async fn get_info_handler(header_map: HeaderMap) -> Result<Json<String>, StatusCode> {
    if let Some(auth_header) = header_map.get("Authorization") {
        if let Ok(auth_header_str) = auth_header.to_str() {
            if auth_header_str.starts_with("Bearer ") {
                let token = auth_header_str.trim_start_matches("Bearer ").to_string();
                {
                    match decode::<Claims>(
                        &token,
                        &DecodingKey::from_secret(jwt_secret().as_ref()),
                        &Validation::default(),
                    ) {
                        Ok(_) => {
                            let info = "Es válido info: ".to_string();
                            return Ok(Json(info));
                        }
                        Err(e) => {
                            eprintln!("Error generating token {}", e);
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::{types::RegisterInfo, validation::register_handler},
        repositories::user::MockUserRepository,
    };
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use axum::{Json, http::StatusCode};
    use mockall::predicate::{always, eq};
    use uuid::Uuid;

    fn register_info(username: &str, email: &str, password: &str) -> RegisterInfo {
        RegisterInfo {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }

    #[tokio::test]
    async fn register_creates_user_and_returns_id() {
        let id = Uuid::new_v4();
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists()
            .with(eq("ada@example.com"))
            .times(1)
            .returning(|_| false);
        repo.expect_username_exists()
            .with(eq("ada"))
            .times(1)
            .returning(|_| false);
        repo.expect_create()
            .with(eq("ada"), eq("ada@example.com"), always())
            .times(1)
            .returning(move |_, _, password| {
                let hash = PasswordHash::new(password).expect("handler must hash the password");
                Argon2::default()
                    .verify_password(b"correct horse battery staple", &hash)
                    .expect("stored hash must verify against the plaintext");
                Ok(id)
            });

        let response = register_handler(
            &mut repo,
            Json(register_info(
                "ada",
                "ada@example.com",
                "correct horse battery staple",
            )),
        )
        .await
        .expect("registration should succeed");

        assert_eq!(response.0.id, id.to_string());
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(1).returning(|_| true);
        repo.expect_username_exists().times(0);
        repo.expect_create().times(0);

        let response = register_handler(
            &mut repo,
            Json(register_info(
                "ada",
                "ada@example.com",
                "correct horse battery staple",
            )),
        )
        .await;

        assert!(matches!(response, Err(StatusCode::CONFLICT)));
    }

    #[tokio::test]
    async fn register_rejects_duplicate_username() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(1).returning(|_| false);
        repo.expect_username_exists().times(1).returning(|_| true);
        repo.expect_create().times(0);

        let response = register_handler(
            &mut repo,
            Json(register_info(
                "ada",
                "ada@example.com",
                "correct horse battery staple",
            )),
        )
        .await;

        assert!(matches!(response, Err(StatusCode::CONFLICT)));
    }

    #[tokio::test]
    async fn register_rejects_invalid_payload_without_touching_db() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(0);
        repo.expect_username_exists().times(0);
        repo.expect_create().times(0);

        let response = register_handler(
            &mut repo,
            Json(register_info(
                "ada",
                "not-an-email",
                "correct horse battery staple",
            )),
        )
        .await;

        assert!(matches!(response, Err(StatusCode::BAD_REQUEST)));
    }
}
