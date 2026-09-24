use async_trait::async_trait;

use crate::db::schema::user::User;
use crate::db::schema::profile::Profile;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&mut self, email: &str) -> Result<User, toasty::Error>;
    async fn email_exists(&mut self, email: &str) -> bool;
    async fn username_exists(&mut self, username: &str) -> bool;
    async fn create(
        &mut self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<(uuid::Uuid, Profile), toasty::Error>;
}

#[derive(Clone)]
pub struct UserRepositoryImpl {
    db: toasty::Db,
}

impl UserRepositoryImpl {
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_email(&mut self, email: &str) -> Result<User, toasty::Error> {
        User::get_by_email(&mut self.db, email).await
    }

    async fn email_exists(&mut self, email: &str) -> bool {
        User::get_by_email(&mut self.db, email).await.is_ok()
    }

    async fn username_exists(&mut self, username: &str) -> bool {
        let users = match User::filter(User::fields().name().eq(username))
            .exec(&mut self.db)
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

    async fn create(
        &mut self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<(uuid::Uuid, Profile), toasty::Error> {
        let result = User::create()
            .name(name)
            .email(email)
            .password(password_hash)
            .profile(Profile::create())
            .exec(&mut self.db)
            .await?;
        
        // Get the created profile
        let profile = Profile::filter(Profile::fields().user_id().eq(result.id))
            .exec(&mut self.db)
            .await?
            .pop()
            .ok_or(toasty::Error::condition_failed("Profile not created"))?;
        
        Ok((result.id, profile))
    }
}