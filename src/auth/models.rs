use serde::Serialize;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginInfo{
    pub username : String,
    pub password : String,

}
#[derive(Serialize)]
pub struct LoginResponse {
    pub token : String
}

#[derive(Serialize,Deserialize)]
pub struct Claims {
    pub sub : String,
    pub exp : usize,
}