use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::result::Result::Ok;

use crate::model::{Claims, LoginInfo, LoginResponse};

pub async fn login_handler(Json(login_info): Json<LoginInfo>) -> Result<Json<LoginResponse>, StatusCode>{
    let username = &login_info.username;
    let password = &login_info.password;
    let is_Valid = is_valid_user(username, password);
    
    if is_Valid {
        let claims = Claims{
            sub : username.clone(),
            exp : (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize
        };
        let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())){
            Ok(tok) => tok,
            Err(e) => {
                eprintln!("Error generating token {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR)
            },

        };
        Ok(Json(LoginResponse { token }))
    }else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn is_valid_user(username : &str, password : &str) -> bool {
    //aca va logica db
    username != "" && password != ""
}

pub async fn get_info_handler(header_map: HeaderMap) -> Result <Json<String>,StatusCode>{
        if let Some(auth_header) = header_map.get("Authorization"){

            if let Ok(auth_header_str) = auth_header.to_str(){
                if auth_header_str.starts_with("Bearer "){
                    let token = auth_header_str.trim_start_matches("Bearer ").to_string();{
                        match decode::<Claims>(&token, &DecodingKey::from_secret("secret".as_ref()), &Validation::default()){
                            Ok(_) => {
                                let info = "Es válido info: ".to_string();
                                return Ok(Json(info));
                            }
                            Err(e) => {
                                eprintln!("Error generating token {}", e);
                                return Err(StatusCode::INTERNAL_SERVER_ERROR)
                            },
                        }
                    }
                }
            }
        }

        Err(StatusCode::UNAUTHORIZED)

}

