use chrono::{DateTime, Utc};

use super::UserID;

pub struct Reply {
    pub created: DateTime<Utc>,
    pub user: UserID,
    pub content: String,
}