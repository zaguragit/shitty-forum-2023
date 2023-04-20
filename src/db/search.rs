use crate::data::{TopicID, ThreadID};

use super::DB;

impl DB {
    pub fn search_topics(&self, query: &str) -> Vec<&TopicID> {
        self.topics.keys()
            .filter(|id| Self::match_title_to_query(id.0.as_str(), query))
            .collect()
    }

    pub fn search_threads(&self, query: &str) -> Vec<&ThreadID> {
        self.threads.iter()
            .filter(|(_, thread)| Self::match_title_to_query(thread.title.as_str(), query))
            .map(|(id, _)| id)
            .collect()
    }

    fn match_title_to_query(title: &str, query: &str) -> bool {
        title.contains(query)
    }
}