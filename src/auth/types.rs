use serde::Serialize;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginInfo{
    pub username : String,
    pub password : String,

}
#[derive(Serialize,ToSchema)]
pub struct LoginResponse {
    pub token : String
}

#[derive(Serialize,Deserialize)]
pub struct Claims {
    pub sub : String,
    pub exp : usize,
}