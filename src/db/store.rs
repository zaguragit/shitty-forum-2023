use std::{collections::HashMap, fs::{read_dir, read_to_string, create_dir_all}};

use chrono::{DateTime, Utc};
use json::{JsonValue, object};
use rand::distributions::{Alphanumeric, DistString};

use crate::{data::{Topic, User, UserID, TopicID, ThreadID, Thread, ReplyID, Reply, ModItemID}, auth::PasswordStore};

use super::Permission;

pub(super) const USERS_PATH: &str = "store/users";
pub(super) const TOPICS_PATH: &str = "store/topics";
pub(super) const THREADS_PATH: &str = "store/threads";
pub(super) const REPLIES_PATH: &str = "store/replies";
pub(super) const AUTH_PATH: &str = "store/auth";
pub(super) const MOD_PATH: &str = "store/mod";
pub(super) const MOD_INSPECTION_PATH: &str = "store/mod/inspection";
pub(super) const MOD_RECORD_PATH: &str = "store/mod/record";

pub(super) fn load_users() -> HashMap<UserID, User> {
    match read_dir(USERS_PATH) {
        Ok(x) => x.map(|x| {
            let file = x.unwrap();
            let name = file.file_name().into_string().unwrap();
            let name = name[0..name.find('.').unwrap_or_else(|| name.len())].to_string();
            let json = json::parse(&read_to_string(file.path()).unwrap()).unwrap();
            let about = json["about"].to_string();
            let pronouns = match &json["pronouns"] {
                JsonValue::Array(pronouns) => Some([
                    pronouns.get(0).map_or_else(|| "null".to_string(), |x| x.to_string()),
                    pronouns.get(1).map_or_else(|| "null".to_string(), |x| x.to_string()),
                    pronouns.get(2).map_or_else(|| "null".to_string(), |x| x.to_string()),
                ]),
                _ => None
            };
            let fav_topics = match &json["fav-topics"] {
                JsonValue::Array(topics) => topics.iter()
                    .map(|x| TopicID(x.as_str().unwrap().to_string()))
                    .collect(),
                _ => vec![],
            };
            let fav_threads = match &json["fav-threads"] {
                JsonValue::Array(threads) => threads.iter()
                    .map(|x| ThreadID(x.as_str().unwrap().to_string()))
                    .collect(),
                _ => vec![],
            };
            (UserID(name.clone()), User { about, pronouns, fav_topics, fav_threads })
        }).collect(),
        Err(_) => HashMap::new(),
    }
}

pub(super) fn load_topics() -> HashMap<TopicID, Topic> {
    match read_dir(TOPICS_PATH) {
        Ok(x) => x.map(|x| {
            let file = x.unwrap();
            let name = file.file_name().into_string().unwrap();
            let name = name[0..name.find('.').unwrap_or_else(|| name.len())].to_string();
            let json = json::parse(&read_to_string(file.path()).unwrap()).unwrap();
            let about = json["about"].to_string();
            let threads = match &json["threads"] {
                JsonValue::Array(threads) => threads.iter()
                    .map(|x| ThreadID(x.as_str().unwrap().to_string()))
                    .collect(),
                _ => vec![],
            };
            (TopicID(name.clone()), Topic { about, threads })
        }).collect(),
        Err(_) => HashMap::new(),
    }
}

pub(super) fn load_threads() -> HashMap<ThreadID, Thread> {
    match read_dir(THREADS_PATH) {
        Ok(x) => x.map(|x| {
            let file: std::fs::DirEntry = x.unwrap();
            let name = file.file_name().into_string().unwrap();
            let name = name[0..name.find('.').unwrap_or_else(|| name.len())].to_string();
            let json = json::parse(&read_to_string(file.path()).unwrap()).unwrap();
            let title = json["title"].to_string();
            let replies = match &json["replies"] {
                JsonValue::Array(replies) => replies.iter()
                    .map(|x| ReplyID(x.as_str().unwrap().to_string()))
                    .collect(),
                _ => vec![],
            };
            (ThreadID(name.clone()), Thread { title, replies })
        }).collect(),
        Err(_) => HashMap::new(),
    }
}

pub(super) fn load_replies() -> HashMap<ReplyID, Reply> {
    match read_dir(REPLIES_PATH) {
        Ok(x) => x.map(|x| {
            let file = x.unwrap();
            let name = file.file_name().into_string().unwrap();
            let name = name[0..name.find('.').unwrap_or_else(|| name.len())].to_string();
            let json = json::parse(&read_to_string(file.path()).unwrap()).unwrap();
            let created = json["created"].as_str().and_then(|x| x.parse::<DateTime<Utc>>().ok()).unwrap();
            let user = UserID(json["user"].to_string());
            let content = json["content"].to_string();
            (ReplyID(name), Reply { created, user, content })
        }).collect(),
        Err(_) => HashMap::new(),
    }
}

pub(super) fn load_permissions() -> HashMap<UserID, Vec<Permission>> {
    let json = read_to_string(MOD_PATH.to_string() + "/permissions.json")
        .ok().and_then(|j| json::parse(&j).ok());
    match json {
        Some(JsonValue::Object(json)) => {
            json.iter().map(|x| {
                let user_id = UserID(x.0.to_string());
                let permissions = if let JsonValue::Array(permissions) = x.1 {
                    permissions.iter().filter_map(|x| {
                        match x.as_str() {
                            Some("overlord") => Some(Permission::Overlord),
                            Some(x) if x.starts_with("mod:") =>
                                Some(Permission::TopicOwner(
                                    TopicID(x.replacen("mod:", "", 1))
                                )),
                            _ => None,
                        }
                    }).collect()
                } else { vec![] };
                (user_id, permissions)
            }).collect::<HashMap<_, _>>()
        },
        _ => HashMap::new(),
    }
}


pub(super) fn store_user(id: &UserID, user: &User) {
    let _ = create_dir_all(USERS_PATH);
    let json = object! {
        about: user.about.as_str(),
        pronouns: user.pronouns.as_ref().and_then(|x| Some(x.as_slice())),
        "fav-topics": user.fav_topics.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
        "fav-threads": user.fav_threads.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
    };
    let _ = std::fs::write(USERS_PATH.to_string() + "/" + id.0.as_str() + ".json", json.to_string());
}

pub(super) fn store_topic(id: &TopicID, topic: &Topic) {
    let _ = create_dir_all(TOPICS_PATH);
    let json = object! {
        about: topic.about.as_str(),
        threads: topic.threads.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
    };
    let _ = std::fs::write(TOPICS_PATH.to_string() + "/" + id.0.as_str() + ".json", json.to_string());
}

pub(super) fn store_thread(id: &ThreadID, thread: &Thread) {
    let _ = create_dir_all(THREADS_PATH);
    let json = object! {
        title: thread.title.as_str(),
        replies: thread.replies.iter().map(|x| x.0.as_str()).collect::<Vec<_>>(),
    };
    let _ = std::fs::write(THREADS_PATH.to_string() + "/" + id.0.as_str() + ".json", json.to_string());
}

pub(super) fn store_reply(id: &ReplyID, reply: &Reply) {
    let _ = create_dir_all(REPLIES_PATH);
    let json = object! {
        created: reply.created.to_string().as_str(),
        user: reply.user.0.as_str(),
        content: reply.content.as_str(),
    };
    let _ = std::fs::write(REPLIES_PATH.to_string() + "/" + id.0.as_str() + ".json", json.to_string());
}

pub(super) fn store_permissions(permissions: &HashMap<UserID, Vec<Permission>>) {
    let _ = create_dir_all(MOD_PATH);
    let mut obj = JsonValue::new_object();
    for (user, permissions) in permissions {
        let string = permissions.iter().map(|permission| {
            let permission = match permission {
                Permission::Overlord => "overlord".to_string(),
                Permission::TopicOwner(topic) =>
                    "mod:".to_string() + topic.0.as_str(),
            };
            JsonValue::String(permission)
        }).collect::<Vec<_>>();
        obj[&user.0] = JsonValue::Array(string);
    }
    let _ = std::fs::write(MOD_PATH.to_string() + "/overlords.json", obj.to_string());
}


pub(super) fn delete_user(id: &UserID) {
    let _ = std::fs::remove_file(USERS_PATH.to_string() + "/" + id.0.as_str() + ".json");
}

pub(super) fn delete_topic(id: &TopicID) {
    let _ = std::fs::remove_file(TOPICS_PATH.to_string() + "/" + id.0.as_str() + ".json");
}

pub(super) fn delete_thread(id: &ThreadID) {
    let _ = std::fs::remove_file(THREADS_PATH.to_string() + "/" + id.0.as_str() + ".json");
}

pub(super) fn delete_reply(id: &ReplyID) {
    let _ = std::fs::remove_file(REPLIES_PATH.to_string() + "/" + id.0.as_str() + ".json");
}

fn gen_id(path: &'static str) -> String {
    let _ = create_dir_all(path);
    let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 24);
    match read_dir(path) {
        Ok(dir) => if dir.into_iter().any(|x| {
            let file = x.unwrap();
            let name = file.file_name().into_string().unwrap();
            let name = name[0..name.find('.').unwrap_or_else(|| name.len())].to_string();
            name == id
        }) {
            gen_id(path)
        } else {
            id
        },
        Err(_) => id,
    }
}

pub(super) fn gen_reply_id() -> ReplyID {
    ReplyID(gen_id(REPLIES_PATH))
}

pub(super) fn gen_thread_id() -> ThreadID {
    ThreadID(gen_id(THREADS_PATH))
}

pub(super) fn gen_inspection_id() -> ModItemID {
    ModItemID(gen_id(MOD_INSPECTION_PATH))
}


pub fn store_user_auth(user_name: &str, password_store: &PasswordStore) {
    let _ = create_dir_all(AUTH_PATH);
    let json = object! {
        salt: password_store.salt.as_str(),
        hashed: password_store.hashed.as_str(),
    };
    let _ = std::fs::write(AUTH_PATH.to_string() + "/" + user_name + ".json", json.to_string());
}

pub fn load_user_auth(user_name: &str) -> Option<PasswordStore> {
    let json = read_to_string(AUTH_PATH.to_string() + "/" + user_name + ".json");
    match json {
        Ok(json) => {
            let json = json::parse(&json).unwrap();
            Some(PasswordStore {
                salt: json["salt"].as_str().unwrap().to_string(),
                hashed: json["hashed"].as_str().unwrap().to_string(),
            })
        },
        Err(_) => None,
    }
}