use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::result::Result::Ok;

use validator::Validate;

use crate::auth::types::Claims;
use crate::db::schema::user::User;

use super::types::{LoginInfo, LoginResponse, RegisterInfo, RegisterResponse};

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET var should be set")
}

pub async fn login_handler(
    mut db: toasty::Db,
    Json(login_info): Json<LoginInfo>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let email: &String = &login_info.email;
    let password = &login_info.password;
    let is_valid: bool = is_valid_user(&mut db, email, password).await;

    if is_valid {
        let claims = Claims {
            sub: email.clone(),
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
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn is_valid_user(executor: &mut dyn toasty::Executor, email: &str, password: &str) -> bool {
    let user = match User::get_by_email(executor, email).await {
        Ok(user) => user,
        Err(_) => return false,
    };
    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub async fn is_duplicate_email(executor: &mut dyn toasty::Executor, email: &str) -> bool {
    matches!(User::get_by_email(executor, email).await, Ok(_))
}
pub async fn is_duplicate_username(executor: &mut dyn toasty::Executor, username: &str) -> bool {
    let users = match User::filter(User::fields().name().eq(username))
        .exec(executor)
        .await
    {
        Ok(users) => users,
        Err(e) => {
            eprintln!("Error checking duplicate username {}", e);
            return false;
        }
    };
    !users.is_empty()
}

pub async fn register_handler(
    mut db: toasty::Db,
    Json(register_info): Json<RegisterInfo>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    register_info.validate().map_err(|_| StatusCode::BAD_REQUEST)?;
    let username: &String = &register_info.username;
    let password: &String = &register_info.password;
    let email: &String = &register_info.email;
    let duplicate_email: bool = is_duplicate_email(&mut db, email).await;
    if duplicate_email {
        return Err(StatusCode::CONFLICT);
    }
    let duplicate_username: bool = is_duplicate_username(&mut db, username).await;
    if duplicate_username {
        return Err(StatusCode::CONFLICT);
    }
    let hash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();


    let user = User::create()
    .name(username)
    .email(email)
    .password(hash)
    .exec(&mut db)
    .await
    .map_err(|e| {
        eprintln!("Error creating user {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    
   Ok(Json(RegisterResponse { id: user.id.to_string() }))

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
