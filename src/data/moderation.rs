use chrono::{DateTime, Utc};

use super::{TopicID, ThreadID, Reply, Topic, User, Thread};

pub struct ModItem {
    pub moderated: DateTime<Utc>,
    pub thing: Moderatable,
}

pub enum Moderatable {
    User(User),
    Topic(User, Topic),
    Thread(User, Thread, TopicID),
    Reply(User, Reply, ThreadID),
}