use crate::db::schema::profile::Profile;
use jiff::Timestamp;

#[derive(Debug, toasty::Model)]
#[table = "users"]
pub struct User {
    #[key]
    #[auto]
    pub id: uuid::Uuid,

    #[column(type = varchar(100))]
    name: String,

    #[unique]
    #[column(type = varchar(320))]
    email: String,

    pub password: String,

    #[has_one]
    profile: toasty::Deferred<Option<Profile>>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}
