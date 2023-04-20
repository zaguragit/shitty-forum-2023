use std::{sync::Mutex, collections::HashSet};

use crate::data::{TopicID, ReplyID};
use crate::{auth::UserSession, data::ThreadID};
use crate::db::DB;
use actix_web::http::StatusCode;
use actix_web::http::header::LOCATION;
use actix_web::web::{Data, Form};
use actix_web::{post, HttpResponse};
use ammonia::Builder;
use regex::Regex;
use thiserror::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MakeReply {
    thread: String,
    content: String,
}

#[derive(Deserialize)]
pub struct MakeThread {
    topic: String,
    title: String,
    first: String,
}

#[derive(Deserialize)]
pub struct FavoriteTopic {
    topic: String,
    favorite: bool,
}

#[derive(Deserialize)]
pub struct FavoriteThread {
    thread: String,
    favorite: bool,
}

#[derive(Deserialize)]
pub struct SettingsForm {
    pub about: String,
    pub pronouns: String,
}

#[derive(Deserialize)]
pub struct DeleteReply {
    thread: String,
    reply: String,
}

#[derive(Deserialize)]
pub struct ModReply {
    thread: String,
    reply: String,
}

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Invalid pronouns format. Must be either nominative/oblique/possessive or empty")]
    InvalidPronounsFormat,
}

fn redirect(to: String, session: &UserSession) -> HttpResponse {
    session.keep(HttpResponse::build(StatusCode::SEE_OTHER)
        .append_header((LOCATION, to)))
        .finish()
}

#[post("/do/reply")]
pub async fn make_reply(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<MakeReply>) -> HttpResponse {
    let content = Builder::new()
        .tags(HashSet::from([
            "a", "abbr", "acronym", "area", "aside", "b", "bdi",
            "bdo", "blockquote", "br", "caption", "cite", "code",
            "col", "colgroup", "data", "dd", "del", "details", "dfn", "div",
            "dl", "dt", "em", "figcaption", "figure", "h1", "h2",
            "h3", "h4", "h5", "h6", "hgroup", "i", "img",
            "ins", "kbd", "li", "map", "mark", "ol", "p", "pre",
            "q", "rp", "rt", "ruby", "s", "samp", "small", "span",
            "strong", "sub", "summary", "sup", "table", "tbody",
            "td", "tfoot", "th", "thead", "time", "tr", "u", "ul", "var", "wbr"
        ]))
        .clean_content_tags(HashSet::from(["script", "style", "iframe"]))
        .clean(input.content.as_str())
        .to_string();
    let content = Regex::new("> *\\n *").unwrap().replace_all(content.as_str(), ">").to_string();
    let content = Regex::new(" *\\n *<").unwrap().replace_all(content.as_str(), "<").to_string();
    let content = Regex::new(" *\\n *").unwrap().replace_all(content.as_str(), "<br>").to_string();
    let content = Regex::new(" +").unwrap().replace_all(content.as_str(), " ").to_string();
    let _ = db.lock().unwrap().try_reply(content.as_str(), &ThreadID(input.thread.clone()), &user.user);
    redirect(format!("/t/{}", input.thread), &user)
}

#[post("/do/thread")]
pub async fn make_thread(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<MakeThread>) -> HttpResponse {
    let content = Builder::new()
        .tags(HashSet::from(["b", "i", "em", "q", "u", "var"]))
        .clean_content_tags(HashSet::from(["script", "style", "iframe"]))
        .clean(input.first.as_str())
        .to_string()
        .replace("\n", "")
        .replace("  ", "");
    let mut db = db.lock().unwrap();
    let Some(id) = db.create_new_thread(&TopicID(input.topic.clone()), input.title.clone()) else {
        return redirect(format!("/λ/{}", input.topic), &user);
    };
    let _ = db.try_reply(content.as_str(), &id, &user.user);
    redirect(format!("/t/{}", id.0), &user)
}

#[post("/do/fav-topic")]
pub async fn favorite_topic(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<FavoriteTopic>) -> HttpResponse {
    let _ = db.lock().unwrap()
        .favorite_topic(&user.user, &TopicID(input.topic.clone()), input.favorite);
    redirect(format!("/λ/{}", input.topic), &user)
}

#[post("/do/fav-thread")]
pub async fn favorite_thread(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<FavoriteThread>) -> HttpResponse {
    let _ = db.lock().unwrap()
        .favorite_thread(&user.user, &ThreadID(input.thread.clone()), input.favorite);
    redirect(format!("/t/{}", input.thread), &user)
}

#[post("/do/update-settings")]
pub async fn update_settings(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<SettingsForm>) -> HttpResponse {
    let pronouns = match input.pronouns.split("/").take(3).collect::<Vec<_>>().as_slice() {
        [""] => Ok(None),
        [x, y, z] => Ok(Some([x.to_string(), y.to_string(), z.to_string()])),
        _ => Err(SettingsError::InvalidPronounsFormat),
    };
    match pronouns {
        Ok(pronouns) => {
            let about = Builder::new()
                .add_tags(HashSet::from(["style"]))
                .clean_content_tags(HashSet::from(["script"]))
                .clean(input.about.as_str())
                .to_string();
            let _ = db.lock().unwrap()
                .update_user(&user.user, about, pronouns);
            redirect(format!("/u/{}", user.user.0), &user)
        },
        Err(e) => redirect(format!("/settings?error={}", e.to_string()), &user)
    }
}

#[post("/do/delete/reply")]
pub async fn delete_reply(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<DeleteReply>) -> HttpResponse {
    let mut db = db.lock().unwrap();
    let reply_id = ReplyID(input.reply.clone());
    let Some(reply) = db.get_reply(&reply_id) else {
        return redirect("/inspection".to_string(), &user);
    };
    if user.user == reply.user {
        db.delete_reply(&ThreadID(input.thread.clone()), &reply_id);
    }
    redirect("/inspection".to_string(), &user)
}

#[post("/do/mod/reply")]
pub async fn move_reply_to_inspection(db: Data<Mutex<DB>>, user: UserSession, Form(input): Form<ModReply>) -> HttpResponse {
    let mut db = db.lock().unwrap();
    if db.is_admin(&user.user) {
        db.move_reply_to_inspection(&ThreadID(input.thread.clone()), &ReplyID(input.reply.clone()));
    }
    redirect("/inspection".to_string(), &user)
}