use std::{fs::read_to_string, collections::HashSet};

use actix_web::{HttpResponse, http::{header::ContentType, StatusCode}};
use ammonia::Builder;

use crate::{db::DB, data::{TopicID, UserID, Reply, Thread, ThreadID}, auth::UserSession};

use self::format::format_date_time;

mod format;

pub fn render_page<R>(db: &DB, user_session: Option<&UserSession>, render_content: R) -> HttpResponse
    where R: FnOnce() -> String {
    let user = user_session.map(|x| &x.user);
    let html = read_to_string("assets/index.html").unwrap();
    let topic_list = match user.and_then(|x| db.get_user(x)) {
        Some(user) => {
            let topics = user.fav_topics.iter()
                .map(|name| 
                    read_to_string("assets/element/side-bar/item.html").unwrap()
                        .replace("{{text}}", name.0.as_str())
                        .replace("{{url}}", (String::from("/λ/") + name.0.as_str()).as_str()))
                .collect::<Vec<_>>().join("");
            read_to_string("assets/element/side-bar/fav-topic-list.html").unwrap()
                .replace("{{topics}}", topics.as_str())
        },
        None => "".to_string(),
    };
    let thread_list = match user.and_then(|x| db.get_user(x)) {
        Some(user) => {
            let threads = user.fav_threads.iter()
                .map(|id| {
                    let title = db.get_thread(id).unwrap();
                    read_to_string("assets/element/side-bar/item.html").unwrap()
                        .replace("{{text}}", title.title.as_str())
                        .replace("{{url}}", (String::from("/t/") + id.0.as_str()).as_str())
                })
                .collect::<Vec<_>>().join("");
            read_to_string("assets/element/side-bar/fav-thread-list.html").unwrap()
                .replace("{{threads}}", threads.as_str())
        },
        None => "".to_string(),
    };
    let admin_tools = match user {
        Some(user) if db.is_admin(user) => read_to_string("assets/element/side-bar/admin-tools.html").unwrap(),
        _ => "".to_string(),
    };
    let html = html
        .replace("{{content}}", render_content().as_str())
        .replace("{{fav-topic-list}}", topic_list.as_str())
        .replace("{{fav-thread-list}}", thread_list.as_str())
        .replace("{{admin-tools}}", admin_tools.as_str())
        .replace("{{session-area}}", match user {
            Some(user) => read_to_string("assets/element/top-bar/logged-in.html").unwrap()
                .replace("{{current-user-name}}", user.0.as_str()),
            None => read_to_string("assets/element/top-bar/logged-out.html").unwrap(),
        }.as_str());
    let mut builder = HttpResponse::build(StatusCode::OK);
    builder.content_type(ContentType::html());
    user_session.map(|x| x.keep(&mut builder));
    builder.body(html)
}

pub fn render_thread(db: &DB, preloaded_html: &str, thread_id: &ThreadID) -> String {
    let thread = db.get_thread(thread_id).unwrap();
    let last_reply = thread.replies.last()
        .and_then(|x| db.get_reply(x)).unwrap();
    let last_reply_content = Builder::new()
        .tags(HashSet::from(["b", "i", "em", "q", "u", "var"]))
        .clean(last_reply.content.as_str())
        .to_string();
    let user = db.get_user(&last_reply.user);
    preloaded_html
        .replace("{{created-time}}", format_date_time(&last_reply.created).as_str())
        .replace("{{thread-id}}", thread_id.0.as_str())
        .replace("{{user-name}}", user.map_or_else(|| "[user not found]", |_| last_reply.user.0.as_str()))
        .replace("{{title}}", thread.title.as_str())
        .replace("{{content}}", last_reply_content.as_str())
}

pub fn render_reply(db: &DB, preloaded_html: &str, reply: &Reply) -> String {
    let user = db.get_user(&reply.user);
    preloaded_html
        .replace("{{created-time}}", format_date_time(&reply.created).as_str())
        .replace("{{pronouns}}", user.and_then(|x| x.pronouns.as_ref()).map_or_else(|| "unknown pronouns".to_string(), |x| x.join("/")).as_str())
        .replace("{{user-name}}", user.map_or_else(|| "[user not found]", |_| reply.user.0.as_str()))
        .replace("{{content}}", reply.content.as_str())
}

pub fn render_user_reply(preloaded_html: &str, thread_id: &ThreadID, thread: &Thread, reply: &Reply) -> String {
    preloaded_html
        .replace("{{created-time}}", format_date_time(&reply.created).as_str())
        .replace("{{thread-id}}", thread_id.0.as_str())
        .replace("{{thread-title}}", &thread.title.as_str())
        .replace("{{content}}", reply.content.as_str())
}

pub fn render_topic_fav(db: &DB, user: Option<&UserID>, topic: &TopicID) -> String {
    match user {
        Some(user) => {
            let is_favorite = db.is_topic_favorite(user, topic);
            read_to_string("assets/element/fav-topic.html").unwrap()
                .replace("{{will-be-favorite}}", (!is_favorite).to_string().as_str())
                .replace("{{topic-name}}", topic.0.as_str())
                .replace("{{favorite-text}}", if is_favorite { "Unmark as Favorite" } else { "Mark as Favorite" })
        },
        None => "".to_string(),
    }
}

pub fn render_thread_fav(db: &DB, user: Option<&UserID>, thread: &ThreadID) -> String {
    match user {
        Some(user) => {
            let is_favorite = db.is_thread_favorite(user, thread);
            read_to_string("assets/element/fav-thread.html").unwrap()
                .replace("{{will-be-favorite}}", (!is_favorite).to_string().as_str())
                .replace("{{thread-id}}", thread.0.as_str())
                .replace("{{favorite-text}}", if is_favorite { "Unmark as Favorite" } else { "Mark as Favorite" })
        },
        None => "".to_string(),
    }
}

pub fn render_inspection_reply(db: &DB, preloaded_html: &str, thread_id: &ThreadID, thread: &Thread, reply: &Reply) -> String {
    let user = db.get_user(&reply.user);
    preloaded_html
        .replace("{{created-time}}", format_date_time(&reply.created).as_str())
        .replace("{{thread-id}}", thread_id.0.as_str())
        .replace("{{thread-title}}", &thread.title.as_str())
        .replace("{{pronouns}}", user.and_then(|x| x.pronouns.as_ref()).map_or_else(|| "unknown pronouns".to_string(), |x| x.join("/")).as_str())
        .replace("{{user-name}}", user.map_or_else(|| "[user not found]", |_| reply.user.0.as_str()))
        .replace("{{content}}", reply.content.as_str())
}