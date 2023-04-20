
mod moderation;
mod reply;
mod thread;
mod topic;
mod user;

pub use moderation::*;
pub use reply::*;
pub use thread::*;
pub use topic::*;
pub use user::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThreadID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReplyID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModItemID(pub String);