use std::{collections::{HashMap, HashSet}};

use chrono::Utc;

use crate::{data::{Topic, User, UserID, TopicID, ThreadID, Thread, ReplyID, Reply, ModItemID, ModItem}, auth::PasswordStore};

pub mod favorite;
pub mod inspection;
pub mod permissions;
pub mod search;
pub mod sequence;
pub mod store;

#[derive(Default)]
pub struct DB {
    topics: HashMap<TopicID, Topic>,
    users: HashMap<UserID, User>,
    threads: HashMap<ThreadID, Thread>,
    replies: HashMap<ReplyID, Reply>,

    permissions: HashMap<UserID, Vec<Permission>>,

    inspection: HashMap<ModItemID, ModItem>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Permission {
    Overlord,
    TopicOwner(TopicID),
}

impl DB {
    pub fn load() -> Self {
        let mut l = Self::default();
        l.reload();
        l
    }

    pub fn reload(&mut self) {
        self.users = store::load_users();
        self.topics = store::load_topics();
        self.threads = store::load_threads();
        self.replies = store::load_replies();
        self.permissions = store::load_permissions();
    }

    pub fn get_topic(&self, name: &TopicID) -> Option<&Topic> {
        self.topics.get(name)
    }

    pub fn get_user(&self, name: &UserID) -> Option<&User> {
        self.users.get(name)
    }

    pub fn get_thread(&self, name: &ThreadID) -> Option<&Thread> {
        self.threads.get(name)
    }

    pub fn get_reply(&self, name: &ReplyID) -> Option<&Reply> {
        self.replies.get(name)
    }
}

impl DB {
    pub fn create_new_topic(&mut self, owner: &UserID, name: &str) -> Result<TopicID, ()> {
        let id = TopicID(name.to_string());
        if self.topics.contains_key(&id) {
            Err(())
        } else {
            let topic = Topic::default();
            store::store_topic(&id, &topic);
            self.topics.insert(id.clone(), topic);
            if let Some(permissions) = self.permissions.get_mut(owner) {
                permissions.push(Permission::TopicOwner(id.clone()));
            } else {
                self.permissions.insert(owner.clone(), vec![Permission::TopicOwner(id.clone())]);
            }
            Ok(id)
        }
    }

    pub fn create_new_user(&mut self, name: &str, password_store: &PasswordStore) -> UserID {
        let id = UserID(name.to_string());
        if self.users.contains_key(&id) {
            panic!("User already exists")
        } else {
            let user = User::default();
            store::store_user(&id, &user);
            store::store_user_auth(name, password_store);
            self.users.insert(id.clone(), user);
            id
        }
    }

    pub fn create_new_thread(&mut self, topic_id: &TopicID, title: String) -> Option<ThreadID> {
        let id = store::gen_thread_id();
        let thread = Thread { title, replies: vec![] };
        self.topics.get_mut(topic_id).map(|topic| {
            store::store_thread(&id, &thread);
            self.threads.insert(id.clone(), thread);
            topic.threads.push(id.clone());
            store::store_topic(topic_id, topic);
            id
        })
    }

    pub fn try_reply(&mut self, content: &str, thread_id: &ThreadID, user: &UserID) -> Option<ReplyID> {
        if self.threads.contains_key(thread_id) && self.users.contains_key(user) {
            let reply = Reply { created: Utc::now(), user: user.clone(), content: content.to_string() };
            let id = store::gen_reply_id();
            store::store_reply(&id, &reply);
            self.replies.insert(id.clone(), reply);
            let thread = self.threads.get_mut(thread_id).unwrap();
            thread.replies.push(id.clone());
            store::store_thread(thread_id, thread);
            Some(id)
        } else {
            None
        }
    }

    pub fn update_user(&mut self, user_id: &UserID, about: String, pronouns: Option<[String; 3]>) {
        let user = self.users.get_mut(user_id).unwrap();
        user.about = about;
        user.pronouns = pronouns;
        store::store_user(user_id, user)
    }

    pub fn delete_reply(&mut self, thread_id: &ThreadID, reply_id: &ReplyID) -> Option<Reply> {
        let Some(thread) = self.threads.get_mut(thread_id) else {
            return None;
        };
        let Some(pos) = thread.replies.iter().position(|x| x == reply_id) else {
            return None;
        };
        thread.replies.remove(pos);
        let Some(reply) = self.replies.remove(reply_id) else {
            return None;
        };
        store::store_thread(thread_id, thread);
        store::delete_reply(reply_id);
        Some(reply)
    }

    pub fn delete_thread(&mut self, topic_id: &TopicID, thread_id: &ThreadID) -> Option<(Thread, HashMap<ReplyID, Reply>)> {
        let Some(topic) = self.topics.get_mut(topic_id) else {
            return None;
        };
        let Some(pos) = topic.threads.iter().position(|x| x == thread_id) else {
            return None;
        };
        topic.threads.remove(pos);
        let Some(thread) = self.threads.remove(thread_id) else {
            return None;
        };
        let mut replies = HashMap::new();
        for reply_id in &thread.replies {
            let Some(reply) = self.replies.remove(&reply_id) else {
                continue;
            };
            replies.insert(reply_id.clone(), reply);
            store::delete_reply(reply_id);
        }
        store::store_topic(topic_id, topic);
        store::delete_thread(thread_id);
        Some((thread, replies))
    }
}