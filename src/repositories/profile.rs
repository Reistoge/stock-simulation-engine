use async_trait::async_trait;

use crate::db::schema::profile::Profile;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn create_for_user(&mut self, user_id: uuid::Uuid) -> Result<Profile, toasty::Error>;
    async fn find_by_user_id(&mut self, user_id: uuid::Uuid) -> Result<Profile, toasty::Error>;
    async fn find_by_id(&mut self, id: uuid::Uuid) -> Result<Profile, toasty::Error>;
}

#[derive(Clone)]
pub struct ProfileRepositoryImpl {
    db: toasty::Db,
}

impl ProfileRepositoryImpl {
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProfileRepository for ProfileRepositoryImpl {
    async fn create_for_user(&mut self, user_id: uuid::Uuid) -> Result<Profile, toasty::Error> {
        Profile::create()
            .user_id(user_id)
            .exec(&mut self.db)
            .await
    }

    async fn find_by_user_id(&mut self, user_id: uuid::Uuid) -> Result<Profile, toasty::Error> {
        Profile::filter(Profile::fields().user_id().eq(user_id))
            .exec(&mut self.db)
            .await?
            .pop()
            .ok_or(toasty::Error::condition_failed("Profile not found"))
    }

    async fn find_by_id(&mut self, id: uuid::Uuid) -> Result<Profile, toasty::Error> {
        Profile::get_by_id(&mut self.db, id).await
    }
}