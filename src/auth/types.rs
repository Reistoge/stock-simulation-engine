use serde::Serialize;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema)]
pub struct LoginInfo{
    pub email : String,
    pub password : String,

}
#[derive(Serialize,ToSchema)]
pub struct LoginResponse {
    pub token : String
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterInfo{
    #[validate(length(min = 1))]
    pub username : String,
    #[validate(email)]
    pub email : String,
    #[validate(length(min = 6))]
    pub password : String,

}

/// JWT claims: `sub` is the email for display, `user_id` is the UUID used for scoping.
#[derive(Serialize,Deserialize)]
pub struct Claims {
    pub sub : String,
    pub exp : usize,
    pub user_id : String,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub id : String,
}