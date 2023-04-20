use super::{DB, store};

use crate::data::{UserID, TopicID, ThreadID};

impl DB {
    pub fn favorite_topic(&mut self, user_id: &UserID, topic: &TopicID, favorite: bool) {
        let user = self.users.get_mut(user_id).unwrap();
        let i = user.fav_topics.iter().rposition(|x| x == topic);
        if favorite {
            if i.is_none() {
                user.fav_topics.push(topic.clone());
            }
        } else if let Some(i) = i {
            user.fav_topics.remove(i);
        }
        store::store_user(user_id, user)
    }

    pub fn favorite_thread(&mut self, user_id: &UserID, thread: &ThreadID, favorite: bool) {
        let user = self.users.get_mut(user_id).unwrap();
        let i = user.fav_threads.iter().rposition(|x| x == thread);
        if favorite {
            if i.is_none() {
                user.fav_threads.push(thread.clone());
            }
        } else if let Some(i) = i {
            user.fav_threads.remove(i);
        }
        store::store_user(user_id, user)
    }

    pub fn is_topic_favorite(&self, user: &UserID, topic: &TopicID) -> bool {
        let user = self.users.get(user).unwrap();
        user.fav_topics.contains(topic)
    }

    pub fn is_thread_favorite(&self, user: &UserID, thread: &ThreadID) -> bool {
        let user = self.users.get(user).unwrap();
        user.fav_threads.contains(thread)
    }
}