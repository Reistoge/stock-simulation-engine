use crate::auth::types::{LoginInfo, LoginResponse, RegisterInfo, RegisterResponse};
use crate::auth::validation::{create_jwt, hash_password, validate_jwt, verify_password};
use crate::repositories::user::UserRepository;
use axum::http::StatusCode;
use validator::Validate;

pub struct UserService<R: UserRepository> {
    repo: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn login(mut self, login_info: LoginInfo) -> Result<LoginResponse, StatusCode> {
        let user = self.repo.find_by_email(&login_info.email).await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let valid = verify_password(&login_info.password, &user.password)?;
        if !valid {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = create_jwt(&login_info.email, &user.id.to_string())?;
        Ok(LoginResponse { token })
    }

    pub async fn register(mut self, register_info: RegisterInfo) -> Result<RegisterResponse, StatusCode> {
        register_info.validate().map_err(|_| StatusCode::BAD_REQUEST)?;

        if self.repo.email_exists(&register_info.email).await {
            return Err(StatusCode::CONFLICT);
        }
        if self.repo.username_exists(&register_info.username).await {
            return Err(StatusCode::CONFLICT);
        }

        let hash = hash_password(&register_info.password)?;

        let (user_id, _profile) = self.repo.create(&register_info.username, &register_info.email, &hash).await
            .map_err(|e| {
                eprintln!("Error creating user {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(RegisterResponse { id: user_id.to_string() })
    }

    /// Returns the stored username for a valid token.
    /// Looks the user up by the email claim so renames are reflected; 404 if the user is gone.
    pub async fn get_info(mut self, token: &str) -> Result<String, StatusCode> {
        let claims = validate_jwt(token)?;
        let user = self
            .repo
            .find_by_email(&claims.sub)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        Ok(user.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::{types::RegisterInfo},
        db::schema::profile::Profile,
        repositories::user::MockUserRepository,
    };
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    use axum::http::StatusCode;
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
                // Create a mock profile
                let profile = Profile {
                    id: Uuid::new_v4(),
                    user_id: Some(id),
                    user: Default::default(),
                    stocks: Default::default(),
                    created_at: Default::default(),
                    updated_at: Default::default(),
                };
                Ok((id, profile))
            });

        let service = UserService::new(repo);
        let response = service.register(register_info(
            "ada",
            "ada@example.com",
            "correct horse battery staple",
        )).await.expect("registration should succeed");

        assert_eq!(response.id, id.to_string());
    }

    #[tokio::test]
    async fn register_rejects_duplicate_email() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(1).returning(|_| true);
        repo.expect_username_exists().times(0);
        repo.expect_create().times(0);

        let service = UserService::new(repo);
        let response = service.register(register_info(
            "ada",
            "ada@example.com",
            "correct horse battery staple",
        )).await;

        assert!(matches!(response, Err(StatusCode::CONFLICT)));
    }

    #[tokio::test]
    async fn register_rejects_duplicate_username() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(1).returning(|_| false);
        repo.expect_username_exists().times(1).returning(|_| true);
        repo.expect_create().times(0);

        let service = UserService::new(repo);
        let response = service.register(register_info(
            "ada",
            "ada@example.com",
            "correct horse battery staple",
        )).await;

        assert!(matches!(response, Err(StatusCode::CONFLICT)));
    }

    #[tokio::test]
    async fn register_rejects_invalid_payload_without_touching_db() {
        let mut repo = MockUserRepository::new();

        repo.expect_email_exists().times(0);
        repo.expect_username_exists().times(0);
        repo.expect_create().times(0);

        let service = UserService::new(repo);
        let response = service.register(register_info(
            "ada",
            "not-an-email",
            "correct horse battery staple",
        )).await;

        assert!(matches!(response, Err(StatusCode::BAD_REQUEST)));
    }
}