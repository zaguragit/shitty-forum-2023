use std::{cmp::Ordering, collections::HashMap};

use super::DB;

use crate::data::{UserID, TopicID, ThreadID, Thread, Reply, ModItemID, ModItem};

impl DB {
    pub fn collect_replies_for_user<T, M>(&self, user_id: &UserID, transform: M) -> Vec<T> where M: Fn(&ThreadID, &Thread, &Reply) -> T {
        self.topics.iter().flat_map(|(_, t)|
            t.threads.iter().filter_map(|k| self.threads.get_key_value(k)).flat_map(|(thread_id, thread)|
                thread.replies.iter().filter_map(|t| self.replies.get(t))
                    .filter(|p| &p.user == user_id)
                    .map(|p| transform(thread_id, thread, p))
            ).collect::<Vec<_>>()
        ).collect()
    }

    pub fn get_sorted_threads(&self, topic: &TopicID) -> Vec<&ThreadID> {
        let mut threads = self.get_topic(topic).unwrap().threads.iter()
            .map(|k| (k, self.get_thread(k).unwrap()))
            .collect::<Vec<_>>();
        threads.sort_unstable_by(|(_, a), (_, b)| {
            let a = a.replies.last().and_then(|x| self.get_reply(x));
            let b = b.replies.last().and_then(|x| self.get_reply(x));
            a.map_or(Ordering::Greater, |a|
                b.map_or(Ordering::Less, |b| 
                    b.created.cmp(&a.created)))
        });
        threads.into_iter().map(|(n, _)| n).collect()
    }

    pub fn get_inspection(&self) -> &HashMap<ModItemID, ModItem> {
        &self.inspection
    }
}