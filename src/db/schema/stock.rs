use crate::db::schema::profile::Profile;
use jiff::Timestamp;
#[derive(Debug, toasty::Model)]
#[table = "stocks"]
pub struct Stock {
    #[key]
    #[auto]
    id: uuid::Uuid,

    #[column(type = varchar(100))]
    name: String,

    #[column(type = varchar(5))]
    ticker: String,

    #[unique]
    profile_id: Option<uuid::Uuid>,
    #[belongs_to]
    profile: toasty::Deferred<Option<Profile>>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}
