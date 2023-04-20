use super::{TopicID, ThreadID};

pub struct User {
    pub about: String,
    pub pronouns: Option<[String; 3]>,
    pub fav_topics: Vec<TopicID>,
    pub fav_threads: Vec<ThreadID>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            about: Default::default(),
            pronouns: None,
            fav_topics: vec![],
            fav_threads: vec![],
        }
    }
}