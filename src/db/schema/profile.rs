use jiff::Timestamp;

use crate::db::schema::{stock::Stock, user::User};

#[derive(Debug, toasty::Model)]
#[table = "profiles"]
pub struct Profile {
    #[key]
    #[auto]
    id: uuid::Uuid,

    #[has_many]
    stocks: toasty::Deferred<Vec<Stock>>,

    #[unique]
    user_id: Option<uuid::Uuid>,
    #[belongs_to]
    user: toasty::Deferred<Option<User>>,

    #[default(Timestamp::now())]
    created_at: Timestamp,
    #[update(Timestamp::now())]
    updated_at: Timestamp,
}
