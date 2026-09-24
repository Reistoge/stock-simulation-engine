use crate::db::schema::profile::Profile;
use jiff::Timestamp;

#[derive(Debug, Clone, toasty::Model)]
#[table = "users"]
pub struct User {
    #[key]
    #[auto]
    pub id: uuid::Uuid,

    #[column(type = varchar(100))]
    pub name: String,

    #[unique]
    #[column(type = varchar(320))]
    pub email: String,

    pub password: String,

    #[has_one]
    pub profile: toasty::Deferred<Option<Profile>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}