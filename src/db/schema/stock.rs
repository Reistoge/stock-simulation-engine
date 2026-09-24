use crate::db::schema::profile::Profile;
use jiff::Timestamp;

#[derive(Debug, Clone, toasty::Model)]
#[table = "stocks"]
pub struct Stock {
    #[key]
    #[auto]
    pub id: uuid::Uuid,

    #[column(type = varchar(100))]
    pub name: String,

    #[column(type = varchar(10))]
    pub ticker: String,

    #[unique]
    pub profile_id: Option<uuid::Uuid>,
    #[belongs_to]
    pub profile: toasty::Deferred<Option<Profile>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}