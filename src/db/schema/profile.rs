use jiff::Timestamp;

use crate::db::schema::{stock::Stock, user::User};

#[derive(Debug, Clone, toasty::Model)]
#[table = "profiles"]
pub struct Profile {
    #[key]
    #[auto]
    pub id: uuid::Uuid,

    #[has_many]
    pub stocks: toasty::Deferred<Vec<Stock>>,

    #[unique]
    pub user_id: Option<uuid::Uuid>,
    #[belongs_to]
    pub user: toasty::Deferred<Option<User>>,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[update(Timestamp::now())]
    pub updated_at: Timestamp,
}